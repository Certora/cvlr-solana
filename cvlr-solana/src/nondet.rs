use solana_program::{account_info::AccountInfo, program_option::COption, pubkey::Pubkey};

mod rt_decls {
    use solana_program::{account_info::AccountInfo, pubkey::Pubkey};
    #[allow(improper_ctypes)]
    extern "C" {
        pub fn CVT_nondet_account_info() -> AccountInfo<'static>;
        pub fn CVT_nondet_pubkey() -> Pubkey;
    }
}

#[cfg(feature = "rt")]
mod rt_impls {
    use solana_program::{account_info::AccountInfo, pubkey::Pubkey};
    use solana_sdk::account::Account;
    use std::boxed::Box;

    #[allow(improper_ctypes, improper_ctypes_definitions)]
    #[no_mangle]
    extern "C" fn CVT_nondet_account_info() -> AccountInfo<'static> {
        let owner: &'static mut _ = Box::leak(Box::new(Pubkey::new_unique()));
        let acc = Account::new(1, 2, owner);
        let pk: &'static mut _ = Box::leak(Box::new(Pubkey::new_unique()));
        let v: &'static mut _ = Box::leak(Box::new((*pk, acc)));
        v.into()
    }
    #[no_mangle]
    pub extern "C" fn CVT_nondet_pubkey() -> Pubkey {
        Pubkey::default()
    }
}

/// Returns an `AccountInfo` produced by the `CVT_nondet_account_info` builtin.
///
/// Prefer [`cvlr_new_account_info`](crate::cvlr_new_account_info). The Prover
/// marks this builtin as deprecated: by default it treats the call as a
/// no-op, so the returned `AccountInfo`, including its pointer fields, is
/// whatever was in memory. It is modeled only when the Prover runs with
/// `-solanaCvtNondetAccountInfo true` and the project's summaries file
/// describes it (as `cvlr_summaries_core.txt` from Certora's Solana template
/// does).
pub fn cvlr_nondet_account_info() -> AccountInfo<'static> {
    unsafe { rt_decls::CVT_nondet_account_info() }
}

/// [`cvlr_nondet_account_info`] moved to the heap and leaked, as a
/// `&'static mut`.
///
/// The same caveats apply; prefer
/// [`cvlr_new_account_info`](crate::cvlr_new_account_info).
pub fn cvlr_nondet_account_info_ref() -> &'static mut AccountInfo<'static> {
    Box::leak(Box::new(cvlr_nondet_account_info()))
}

/// An arbitrary `Pubkey`.
///
/// Every call returns an independent value, which may equal any other key.
/// The Prover models the underlying `CVT_nondet_pubkey` builtin through the
/// project's summaries file (`cvlr_summaries_core.txt` in Certora's Solana
/// template), which makes all 32 bytes arbitrary. With the `rt` feature it
/// returns `Pubkey::default()`.
pub fn cvlr_nondet_pubkey() -> Pubkey {
    unsafe { rt_decls::CVT_nondet_pubkey() }
}

/// Either `None` or `Some` of an arbitrary [`cvlr_nondet_pubkey`].
pub fn cvlr_nondet_option_pubkey() -> Option<Pubkey> {
    cvlr_nondet::nondet_option(cvlr_nondet_pubkey)
}

/// Either `COption::None` or `COption::Some(func())`, chosen arbitrarily.
///
/// The `COption` counterpart of `cvlr::nondet::nondet_option`, for the
/// optional fields of SPL Token state (e.g. a mint's authorities or an
/// account's delegate). `func` is called only on the `Some` path.
pub fn cvlr_nondet_coption<T, F>(func: F) -> COption<T>
where
    F: FnOnce() -> T,
{
    if cvlr_nondet::nondet::<bool>() {
        COption::Some(func())
    } else {
        COption::None
    }
}

/// Either `COption::None` or `COption::Some` of an arbitrary
/// [`cvlr_nondet_pubkey`].
pub fn cvlr_nondet_coption_pubkey() -> COption<Pubkey> {
    cvlr_nondet_coption(cvlr_nondet_pubkey)
}
