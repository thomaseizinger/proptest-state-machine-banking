use proptest::{
    arbitrary::any,
    sample,
    strategy::{Just, Strategy},
};
use proptest_state_machine::{ReferenceStateMachine, StateMachineTest};
use proptest_state_machine_banking::Bank;

/// Holds the state of our "simulated" bank.
///
/// This implements the expected logic using a simplified model and is used to check correctness of the real code.
#[derive(Debug, Clone)]
struct SimBank {
    accounts: Vec<SimAccount>,
}

/// A simulated account.
///
/// This holds - contrary to our production code - the actual balance.
/// This is a much simpler model to assert against.
#[derive(Debug, Clone, Default)]
struct SimAccount {
    balance: u64,
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
    Freeze {
        account_id: sample::Index,
    },
    Unfreeze {
        account_id: sample::Index,
    },
    Close {
        account_id: sample::Index,
    },
}

impl ReferenceStateMachine for SimBank {
    type State = Self;
    type Transition = Transition;

    fn init_state() -> proptest::prelude::BoxedStrategy<Self::State> {
        Just(Self {
            accounts: Default::default(),
        })
        .boxed()
    }

    fn transitions(_: &Self::State) -> proptest::prelude::BoxedStrategy<Self::Transition> {
        proptest::prop_oneof![
            Just(Transition::Open),
            (any::<sample::Index>(), any::<u64>())
                .prop_map(|(account_id, amount)| Transition::Deposit { account_id, amount }),
        ]
        .boxed()
    }

    fn apply(mut state: Self::State, transition: &Self::Transition) -> Self::State {
        match transition {
            Transition::Open => state.accounts.push(SimAccount::default()),
            Transition::Withdraw { account_id, amount } => todo!(),
            Transition::Deposit { account_id, amount } => {
                let account = account_id.get_mut(&mut state.accounts);

                account.balance += *amount;
            }
            Transition::Transfer { from, to, amount } => todo!(),
            Transition::Freeze { account_id } => todo!(),
            Transition::Unfreeze { account_id } => todo!(),
            Transition::Close { account_id } => todo!(),
        }

        state
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
            Transition::Freeze { account_id } => {
                let account = account_id.get(&state.open_accounts);

                state.inner.freeze(*account).unwrap();
            }
            Transition::Unfreeze { account_id } => {
                let account = account_id.get(&state.open_accounts);

                state.inner.unfreeze(*account).unwrap();
            }
            Transition::Close { account_id } => {
                let account = account_id.get(&state.open_accounts);

                state.inner.close(*account).unwrap();
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

            assert_eq!(actual_balance, expected_balance);
        }

        state
    }
}
