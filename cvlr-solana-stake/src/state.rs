//! State related functions
//!
//! Reading and writing `StakeStateV2` in account data, and arbitrary stake
//! state.

use cvlr_nondet::havoc::alloc_ref_havoced;
use solana_program::{
    account_info::AccountInfo,
    borsh1::try_from_slice_unchecked,
    stake::{
        stake_flags::StakeFlags,
        state::{Meta, Stake, StakeStateV2},
    },
};

use crate::utils::CvlrStdIoWrite;

/// Deserializes a `StakeStateV2` (borsh) from the start of `data`.
///
/// Trailing bytes are ignored. Panics, ending the execution, if the bytes are
/// not a valid `StakeStateV2`, so executions in which the data is not valid
/// stake state are dropped from the analysis.
#[inline(always)]
pub fn stake_from_slice_unchecked(data: &[u8]) -> StakeStateV2 {
    try_from_slice_unchecked::<StakeStateV2>(data).unwrap()
}

/// [`stake_from_slice_unchecked`] on the data of `acc`.
///
/// Also panics if the account data is currently borrowed mutably.
#[inline(always)]
pub fn stake_from_account_info_unchecked(acc: &AccountInfo) -> StakeStateV2 {
    stake_from_slice_unchecked(&acc.data.borrow()[..])
}

/// Serializes `stake` (borsh) into the start of `data`.
///
/// Bytes after the serialized value are left unchanged. Assumes the value
/// fits in `data` (see [`CvlrStdIoWrite`]).
#[inline(always)]
pub fn stake_to_slice_unchecked(data: &mut [u8], stake: &StakeStateV2) {
    borsh::to_writer(CvlrStdIoWrite(data), stake).unwrap();
}

/// An arbitrary `Meta`: rent-exempt reserve, authorities and lockup are all
/// unconstrained.
pub fn nondet_meta() -> Meta {
    *alloc_ref_havoced::<Meta>()
}

/// An arbitrary `Stake`: delegation (voter, amount, activation and
/// deactivation epochs) and observed credits are all unconstrained.
pub fn nondet_stake() -> Stake {
    *alloc_ref_havoced::<Stake>()
}

/// `StakeFlags::empty()`, the flags of a stake with no flag set.
pub fn empty_stake_flags() -> StakeFlags {
    StakeFlags::empty()
}
