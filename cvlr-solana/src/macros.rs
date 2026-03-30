// We redefine this macro to avoid including error code conversion/formatting
/// Verification replacement for Anchor's `require_keys_eq!`: *assumes* the two
/// keys are equal.
///
/// `require_keys_eq!(a, b)` and `require_keys_eq!(a, b, error_code)` expand to
/// `cvt::CVT_assume(a == b)`; the error code is ignored. Where Anchor's macro
/// returns an error when the keys differ, this one drops those executions
/// from the analysis, so rules only consider executions in which the check
/// passes.
///
/// The expansion names `cvt::CVT_assume`, a function of the legacy `cvt`
/// crate, so the calling crate must have `cvt::CVT_assume(bool)` in scope. A
/// shim is enough:
///
/// ```ignore
/// mod cvt {
///     pub use cvlr::asserts::cvlr_assume_checked as CVT_assume;
/// }
/// ```
#[macro_export]
macro_rules! require_keys_eq {
    ($value1: expr, $value2: expr, $error_code:expr $(,)?) => {
        cvt::CVT_assume($value1 == $value2);
    };
    ($value1: expr, $value2: expr $(,)?) => {
        cvt::CVT_assume($value1 == $value2);
    };
}

/// Verification replacement for Anchor's `require_keys_neq!`: *assumes* the
/// two keys differ.
///
/// `require_keys_neq!(a, b)` and `require_keys_neq!(a, b, error_code)` expand
/// to `cvt::CVT_assume(a != b)`; the error code is ignored. See
/// [`require_keys_eq!`](crate::require_keys_eq) for the consequences and for
/// the `cvt::CVT_assume` the calling crate must provide.
#[macro_export]
macro_rules! require_keys_neq {
    ($value1: expr, $value2: expr, $error_code:expr $(,)?) => {
        cvt::CVT_assume($value1 != $value2);
    };
    ($value1: expr, $value2: expr $(,)?) => {
        cvt::CVT_assume($value1 != $value2);
    };
}

// We redefine invoke and invoke_signed to avoid the signer checks
// TODO: we might want to move these macros to mocks
/// Program-side replacement for `solana_cpi::invoke`.
///
/// `invoke!(instruction, account_infos)` expands to
/// `solana_cpi::invoke_signed_unchecked(instruction, account_infos, &[])`,
/// which skips the runtime's signer and privilege checks. Use it in the code
/// under verification (typically behind the verification feature) in place of
/// `invoke(...)`.
///
/// **This macro does not model the called program.** The standard Prover
/// summaries (`cvlr_summaries_core.txt` in Certora's Solana template) treat
/// `invoke_signed_unchecked` as returning an arbitrary result, `Ok` or an
/// error, with no effect on any account. The callee's effects, such as a token
/// transfer, are not represented. Because the call no longer goes through
/// `invoke`, the Prover's CPI substitution (see the `cvlr-spl-token` crate)
/// does not apply to it either. To model a callee, call a model of it
/// directly (e.g. `cvlr_solana::token::spl_token_transfer`) or mock the
/// wrapper function with `#[cvlr::mock_fn]`.
#[macro_export]
macro_rules! invoke {
    ($instruction: expr, $acc_infos: expr $(,)*) => {{
        $crate::__solana_cpi::invoke_signed_unchecked($instruction, $acc_infos, &[])
    }};
}

/// Program-side replacement for `solana_cpi::invoke_signed`.
///
/// `invoke_signed!(instruction, account_infos, signer_seeds)` expands to
/// `solana_cpi::invoke_signed_unchecked(instruction, account_infos, signer_seeds)`.
/// As with [`invoke!`](crate::invoke), the called program is **not modeled**:
/// the call returns an arbitrary result and changes no account under the
/// standard summaries, and the Prover's CPI substitution does not apply.
#[macro_export]
macro_rules! invoke_signed {
    ($instruction: expr, $acc_infos: expr, $seeds: expr $(,)*) => {{
        $crate::__solana_cpi::invoke_signed_unchecked($instruction, $acc_infos, $seeds)
    }};
}
