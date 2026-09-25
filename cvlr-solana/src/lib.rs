//! Solana support for CVLR, the Certora Verification Language for Rust.
//!
//! Helpers for writing Certora Prover rules about Solana programs:
//!
//! * **Accounts.** [`cvlr_deserialize_nondet_accounts`] returns 16
//!   arbitrary `AccountInfo`s, the usual input of a rule; pass them to the
//!   program's processor. [`cvlr_new_account_info`] creates one.
//! * **Nondeterministic values.** [`cvlr_nondet_pubkey`],
//!   [`cvlr_nondet_option_pubkey`], [`cvlr_nondet_coption`],
//!   [`cvlr_nondet_coption_pubkey`].
//! * **Logging.** [`cvlr_clog_account_info`] / [`clog_acc_info!`] show an
//!   account in the counterexample; [`pubkey::Pk`] makes a `Pubkey` loggable
//!   with `clog!`.
//! * **Models of the environment.** [`token`] reads and writes SPL Token
//!   account and mint fields and models Token instructions;
//!   [`cvt_get_next_clock_slot`] models the clock slot; [`invoke!`] and
//!   [`invoke_signed!`] replace CPIs; [`require_keys_eq!`] and
//!   [`require_keys_neq!`] replace Anchor's checks.
//!
//! SPL Token CPI mocks are in the `cvlr-spl-token` crate and a model of the
//! Stake program is in `cvlr-solana-stake`.
//!
//! Several functions here rely on the Prover summaries in
//! `cvlr_summaries_core.txt`, shipped with Certora's Solana project template;
//! those are noted where they apply.

mod clock;
mod layout;
mod log;
mod macros;
mod nondet;
pub mod pubkey;

pub mod token;

pub use clock::*;
pub use layout::{
    cvlr_deserialize_nondet_accounts, cvlr_deserialize_nondet_accounts as cvlr_nondet_acc_infos,
    cvlr_new_account_info, fun_acc_infos_with_mem_layout,
};
pub use log::*;
pub use nondet::*;
