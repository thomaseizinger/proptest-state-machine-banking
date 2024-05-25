use std::{io, ops::Neg};

/// Implements our "real" bank.
#[derive(Debug, Clone, Default)]
pub struct Bank {
    accounts: Vec<u64>,
    transactions: Vec<Transaction>,
}

impl Bank {
    pub fn open(&mut self, _can_overdraw: bool) -> io::Result<u64> {
        let new_account = self.accounts.last().map_or(0, |id| *id + 1);
        self.accounts.push(new_account);

        Ok(new_account)
    }

    pub fn deposit(&mut self, id: u64, amount: u64) -> io::Result<()> {
        self.transactions
            .push(Transaction::Deposit { to: id, amount });

        Ok(())
    }

    pub fn withdraw(&mut self, id: u64, amount: u64) -> io::Result<()> {
        self.transactions
            .push(Transaction::Withdraw { from: id, amount });

        Ok(())
    }

    pub fn transfer(&mut self, from: u64, to: u64, amount: u64) -> io::Result<()> {
        self.transactions
            .push(Transaction::Transfer { from, to, amount });

        Ok(())
    }

    pub fn balance(&self, id: u64) -> io::Result<i64> {
        let balance = self
            .transactions
            .iter()
            .filter_map(|t| match t {
                Transaction::Deposit { to, amount } => (*to == id).then_some(*amount as i64),
                Transaction::Withdraw { from, amount } => {
                    (*from == id).then_some((*amount as i64).neg())
                }
                Transaction::Transfer { from, to, amount } => {
                    if *from == id {
                        return Some((*amount as i64).neg());
                    }

                    if *to == id {
                        return Some(*amount as i64);
                    }

                    None
                }
            })
            .sum();

        Ok(balance)
    }
}

/// A transaction within our bank.
///
/// It is typical for ledgers to implemented as a series of transactions.
#[derive(Debug, Clone)]
enum Transaction {
    Deposit { to: u64, amount: u64 },
    Withdraw { from: u64, amount: u64 },
    Transfer { from: u64, to: u64, amount: u64 },
}
