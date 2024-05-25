use std::io;

/// Implements our "real" bank.
///
/// A bank is essentially a ledger. Ledgers are typically implemented using a transaction system, i.e. balances are only aggregated from all transactions on-demand.
#[derive(Debug, Clone, Default)]
pub struct Bank {}

impl Bank {
    pub fn deposit(&mut self, id: u64, amount: u64) -> io::Result<()> {
        todo!()
    }

    pub fn withdraw(&mut self, id: u64, amount: u64) -> io::Result<()> {
        todo!()
    }

    pub fn transfer(&mut self, from: u64, to: u64, amount: u64) -> io::Result<()> {
        todo!()
    }

    pub fn close(&mut self, id: u64) -> io::Result<()> {
        todo!()
    }

    pub fn open(&mut self, can_overdraw: bool) -> io::Result<u64> {
        todo!()
    }

    pub fn unfreeze(&mut self, id: u64) -> io::Result<()> {
        todo!()
    }

    pub fn freeze(&mut self, id: u64) -> io::Result<()> {
        todo!()
    }

    pub fn balance(&self, id: u64) -> io::Result<u64> {
        todo!()
    }
}
