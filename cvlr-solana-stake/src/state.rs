//! State related functions

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

#[inline(always)]
pub fn stake_from_slice_unchecked(data: &[u8]) -> StakeStateV2 {
    try_from_slice_unchecked::<StakeStateV2>(data).unwrap()
}

#[inline(always)]
pub fn stake_from_account_info_unchecked(acc: &AccountInfo) -> StakeStateV2 {
    stake_from_slice_unchecked(&acc.data.borrow()[..])
}

#[inline(always)]
pub fn stake_to_slice_unchecked(data: &mut [u8], stake: &StakeStateV2) {
    borsh::to_writer(CvlrStdIoWrite(data), stake).unwrap();
}

pub fn nondet_meta() -> Meta {
    *alloc_ref_havoced::<Meta>()
}

pub fn nondet_stake() -> Stake {
    *alloc_ref_havoced::<Stake>()
}

pub fn empty_stake_flags() -> StakeFlags {
    StakeFlags::empty()
}
