use core::ptr;
use solana_account::Account;
use std::sync::{Mutex, OnceLock};

#[derive(Debug)]
pub struct InstructionAccounts {
    accounts: Vec<Account>,
    next_idx: usize,
}

static GLOBAL: OnceLock<Mutex<InstructionAccounts>> = OnceLock::new();

impl InstructionAccounts {
    // we assume this is single-threaded.
    pub fn init_global(accounts: Vec<Account>) {
        let database = InstructionAccounts {
            accounts,
            next_idx: 0,
        };
        let guard = Mutex::new(database);

        // ybd: do we want to allow re-init?
        GLOBAL.set(guard).expect("can only be set once");
    }

    pub fn global<'a>() -> Option<&'a Mutex<InstructionAccounts>> {
        GLOBAL.get()
    }

    pub fn next_account(&mut self) -> Option<&mut Account> {
        let account = self.accounts.get_mut(self.next_idx)?;
        self.next_idx += 1;
        Some(account)
    }

    pub fn account(&self, idx: usize) -> Option<&Account> {
        self.accounts.get(idx)
    }

    pub fn account_mut(&mut self, idx: usize) -> Option<&mut Account> {
        if self.next_idx > idx {
            panic!("can't get mutable reference to account after it has been allocated")
        } else {
            self.accounts.get_mut(idx)
        }
    }
}
