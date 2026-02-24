use core::ptr;
use solana_sdk::account::Account;
use std::sync::{Mutex, OnceLock};

#[derive(Debug)]
pub struct GlobalDatabase {
    accounts: Vec<Account>,
}
// we assume this is single-threaded.
pub fn init(accounts: Vec<Account>) {
    let database = GlobalDatabase { accounts };
    let guard = Mutex::new(database);

    // ybd: do we want to allow re-init?
    GLOBAL_DATABASE.set(guard).expect("can only be set once");
}

pub static GLOBAL_DATABASE: OnceLock<Mutex<GlobalDatabase>> = OnceLock::new();

impl GlobalDatabase {
    pub fn get(&self, idx: usize) -> Option<&Account> {
        self.accounts.get(idx)
    }

    pub(crate) fn account_ptr(&mut self, idx: usize) -> Option<ptr::NonNull<u8>> {
        let account = self.accounts.get_mut(idx)?;
        let ptr = ptr::NonNull::from_mut(account).cast();
        Some(ptr)
    }
}
