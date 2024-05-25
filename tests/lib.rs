use proptest::{
    arbitrary::any,
    sample,
    strategy::{Just, Strategy},
    test_runner::Config,
};
use proptest_state_machine::{prop_state_machine, ReferenceStateMachine, StateMachineTest};
use proptest_state_machine_banking::Bank;

prop_state_machine! {
    #![proptest_config(Config {
        // Enable verbose mode to make the state machine test print the
        // transitions for each case.
        verbose: 1,
        .. Config::default()
    })]

    #[test]
    fn run_bank_test(
        // This is a macro's keyword - only `sequential` is currently supported.
        sequential
        // The number of transitions to be generated for each case. This can
        // be a single numerical value or a range as in here.
        1..20 => RealBank
    );
}

/// Holds the state of our "simulated" bank.
///
/// This implements the expected logic using a simplified model and is used to check correctness of the real code.
#[derive(Debug, Clone)]
struct SimBank {
    accounts: Vec<SimAccount>,

    next_account_id: u64,
}

/// A simulated account.
///
/// This holds - contrary to our production code - the actual balance.
/// This is a much simpler model to assert against.
#[derive(Debug, Clone)]
struct SimAccount {
    id: u64,
    balance: i64,
}

/// Holds the state of a "real" bank.
///
/// This references our production code.
#[derive(Debug, Clone)]
struct RealBank {
    inner: Bank,

    open_accounts: Vec<u64>,
}

#[derive(Debug, Clone)]
enum Transition {
    Open,
    Withdraw {
        account_id: sample::Index,
        amount: u64,
    },
    Deposit {
        account_id: sample::Index,
        amount: u64,
    },
    Transfer {
        from: sample::Index,
        to: sample::Index,
        amount: u64,
    },
    // Freeze {
    //     account_id: sample::Index,
    // },
    // Unfreeze {
    //     account_id: sample::Index,
    // },
    // Close {
    //     account_id: sample::Index,
    // },
}

impl ReferenceStateMachine for SimBank {
    type State = Self;
    type Transition = Transition;

    fn init_state() -> proptest::prelude::BoxedStrategy<Self::State> {
        Just(Self {
            accounts: Default::default(),
            next_account_id: 0,
        })
        .boxed()
    }

    fn transitions(_: &Self::State) -> proptest::prelude::BoxedStrategy<Self::Transition> {
        proptest::prop_oneof![
            Just(Transition::Open),
            (any::<sample::Index>(), any::<u64>())
                .prop_map(|(account_id, amount)| { Transition::Deposit { account_id, amount } }),
            (any::<sample::Index>(), any::<u64>())
                .prop_map(|(account_id, amount)| { Transition::Withdraw { account_id, amount } }),
            (any::<sample::Index>(), any::<sample::Index>(), any::<u64>())
                .prop_map(|(from, to, amount)| Transition::Transfer { from, to, amount }),
        ]
        .boxed()
    }

    fn apply(mut state: Self::State, transition: &Self::Transition) -> Self::State {
        match transition {
            Transition::Open => {
                state.accounts.push(SimAccount {
                    balance: 0,
                    id: state.next_account_id,
                });
                state.next_account_id += 1;
            }
            Transition::Withdraw { account_id, amount } => {
                account_id.get_mut(&mut state.accounts).balance -= (*amount) as i64;
            }
            Transition::Deposit { account_id, amount } => {
                account_id.get_mut(&mut state.accounts).balance += (*amount) as i64;
            }
            Transition::Transfer { from, to, amount } => {
                from.get_mut(&mut state.accounts).balance -= (*amount) as i64;
                to.get_mut(&mut state.accounts).balance += (*amount) as i64;
            }
        }

        state
    }

    fn preconditions(state: &Self::State, transition: &Self::Transition) -> bool {
        match transition {
            Transition::Open => true,
            Transition::Withdraw { .. } => !state.accounts.is_empty(),
            Transition::Deposit { .. } => !state.accounts.is_empty(),
            Transition::Transfer { from, to, .. } => {
                let have_accounts = !state.accounts.is_empty();

                if !have_accounts {
                    return false;
                }

                from.get(&state.accounts).id != to.get(&state.accounts).id
            }
        }
    }
}

impl StateMachineTest for RealBank {
    type SystemUnderTest = Self;
    type Reference = SimBank;

    fn init_test(_: &<Self::Reference as ReferenceStateMachine>::State) -> Self::SystemUnderTest {
        Self {
            inner: Bank::default(), // If our initial state is not just empty, we can initialize it here from the randomly sampled `SimBank`.
            open_accounts: Default::default(),
        }
    }

    fn apply(
        mut state: Self::SystemUnderTest,
        ref_state: &<Self::Reference as ReferenceStateMachine>::State,
        transition: <Self::Reference as ReferenceStateMachine>::Transition,
    ) -> Self::SystemUnderTest {
        // Apply transition. This is the "act" part of our test.
        match transition {
            Transition::Open => {
                let id = state.inner.open(false).unwrap();

                state.open_accounts.push(id);
            }
            Transition::Withdraw { account_id, amount } => {
                let account = account_id.get(&state.open_accounts);

                state.inner.withdraw(*account, amount).unwrap();
            }
            Transition::Deposit { account_id, amount } => {
                let account = account_id.get(&state.open_accounts);

                state.inner.deposit(*account, amount).unwrap();
            }
            Transition::Transfer { from, to, amount } => {
                let from = from.get(&state.open_accounts);
                let to = to.get(&state.open_accounts);

                state.inner.transfer(*from, *to, amount).unwrap();
            }
        }

        // Assert our state.
        // All accounts should always have the expected balance.
        // (Implementation detail: We rely on the `Vec`s having the same order.)
        let open_accounts = state.open_accounts.iter();
        let sim_accounts = ref_state.accounts.iter();

        for (id, sim_account) in open_accounts.zip(sim_accounts) {
            let actual_balance = state.inner.balance(*id).unwrap();
            let expected_balance = sim_account.balance;

            assert_eq!(
                actual_balance, expected_balance,
                "balance mismatch on account {id}"
            );
        }

        state
    }
}
