use core::ptr;
use solana_sdk::account::Account;
use std::sync::{Mutex, OnceLock};

#[derive(Debug)]
pub struct InstructionAccounts {
    accounts: Vec<Account>,
}
// we assume this is single-threaded.
pub fn init(accounts: Vec<Account>) {
    let database = InstructionAccounts { accounts };
    let guard = Mutex::new(database);

    // ybd: do we want to allow re-init?
    INSTRUCTION_ACCOUNTS
        .set(guard)
        .expect("can only be set once");
}

pub static INSTRUCTION_ACCOUNTS: OnceLock<Mutex<InstructionAccounts>> = OnceLock::new();

impl InstructionAccounts {
    pub fn get(&self, idx: usize) -> Option<&Account> {
        self.accounts.get(idx)
    }

    pub fn account_mut(idx: usize) -> Option<ptr::NonNull<u8>> {
        let guard = INSTRUCTION_ACCOUNTS.get()?;
        let mut db = guard.try_lock().ok()?;
        let account = db.accounts.get_mut(idx)?;
        let ptr = ptr::NonNull::from_mut(account).cast();
        Some(ptr)
    }
}
