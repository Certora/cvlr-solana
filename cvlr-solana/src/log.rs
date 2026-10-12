use cvlr_log::clog;
use solana_program::account_info::AccountInfo;

/// Logs an account to the counterexample call trace, tagged with the source
/// text of the argument.
///
/// `clog_acc_info!(acc)` is
/// [`cvlr_clog_account_info`](crate::cvlr_clog_account_info)`(stringify!(acc),
/// acc)`; `acc` must be a `&AccountInfo`.
#[macro_export]
macro_rules! clog_acc_info {
    ($acc: expr) => {
        $crate::cvlr_clog_account_info(stringify!($acc), $acc);
    };
}

/// Logs the main fields of an account to the counterexample call trace.
///
/// Emits `tag` as a message line, followed by the account's lamports, data
/// length, `is_signer` and `is_writable` (in that order, each under its own
/// label). The key, owner and data contents are not logged; log the key with
/// `clog!(Pk(acc.key))` (see [`Pk`](crate::pubkey::Pk)) if needed. As with
/// all logging, `tag` must be a string literal after inlining (the
/// [`clog_acc_info!`](crate::clog_acc_info) macro passes one).
#[inline(always)]
pub fn cvlr_clog_account_info(tag: &str, acc_info: &AccountInfo) {
    clog!(tag);
    clog!(acc_info.lamports());
    clog!(acc_info.data_len());
    clog!(acc_info.is_signer);
    clog!(acc_info.is_writable);
}
