//! State related functions

use solana_program::{
    account_info::AccountInfo, borsh1::try_from_slice_unchecked, stake::state::StakeStateV2,
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

