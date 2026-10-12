//! Model fo Stake Program
//!
//! A simplified model of the native Stake program, for verifying programs that
//! call it (e.g. stake pools). The [`processor`] functions take the same
//! accounts as the corresponding Stake instructions and update the stake
//! accounts' state and lamports roughly as the real program does. Specs
//! typically call them from a mock of the program's CPI into the Stake program.
//!
//! The model is deliberately abstract:
//!
//! * No signer, authority, custodian or lockup checks are made, and the clock
//!   and stake-history sysvars are not read beyond the clock's epoch.
//! * Effective (activated) stake is an arbitrary amount no greater than the
//!   delegated stake, instead of being computed from the stake history.
//! * The minimum delegation is 1 lamport ([`get_minimum_delegation`]).
//! * The error cases the model does check (and invalid account data) panic
//!   instead of returning an error. A panic ends the execution, so rules see
//!   only the executions in which the instruction succeeds, and each
//!   `process_*` function returns `Ok(())` whenever it returns.
//! * Where an instruction reads the clock, the clock account must be the
//!   `Clock` sysvar with valid contents; other executions are dropped.
//!
//! [`state`] reads and writes `StakeStateV2` in account data.
pub mod processor;
pub mod state;
pub mod tools;
pub mod utils;

// const PERPETUAL_NEW_WARMUP_COOLDOWN_RATE_EPOCH: Option<u64> = Some(0);

/// The minimum delegation used by the model: always 1 lamport.
///
/// The real minimum depends on the cluster's feature set (1 SOL on
/// mainnet-beta); the model uses the smallest value so that no delegation is
/// ruled out.
#[inline]
pub fn get_minimum_delegation() -> u64 {
    tools::get_minimum_delegation().unwrap()
}
