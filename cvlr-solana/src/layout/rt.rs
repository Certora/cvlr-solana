use crate::layout::common::sizes;
use crate::layout::instruction_accounts::InstructionAccounts;
use core::cell::RefCell;
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint;
use solana_program::pubkey::Pubkey;
use std::rc::Rc;

/// fetches the next unallocated account and reads it to [`AccountInfo`]
pub fn cvlr_new_account_info<'a>() -> AccountInfo<'a> {
    let ptr = InstructionAccounts::next_ptr().expect("next account has not been allocated yet");
    unsafe { cvlr_new_account_info_rt(ptr) }
}

unsafe fn cvlr_new_account_info_rt<'a>(input: *mut u8) -> AccountInfo<'a> {
    let mut offset: usize = 0;

    match *(input.add(offset) as *const u8) {
        entrypoint::NON_DUP_MARKER => {
            offset += sizes::NON_DUP_MARKER;
        }
        _ => {
            // TODO: "probably not hard to support. ok to not have now"
            panic!("acccount detected as duplicate")
        }
    };

    let is_signer = *(input.add(offset) as *const u8) != 0;
    offset += sizes::IS_SIGNER;

    let is_writable = *(input.add(offset) as *const u8) != 0;
    offset += sizes::IS_WRITABLE;

    // ybd: for whatever reason, `executable` is here,
    // and not at the end like in entrypoint_deprecated
    let executable = *(input.add(offset) as *const u8) != 0;
    offset += sizes::EXECUTABLE;

    // this is where solana stores the "original length"
    // of the account data, aka the data len in the previous
    // the deserialization.
    // it is used to detect invalid reallocations.
    // it is stored in 4 bytes that were originally just padding.
    //
    // we're going to overwrite this later with the current length.
    // to avoid holding a mutable pointer here we store just the offset
    // so we can write to it later.
    let original_data_len_offset = offset;
    offset += sizes::ORIGINAL_DATA_LEN;

    let key = &*(input.add(offset) as *const Pubkey);
    offset += sizes::KEY;

    let owner = &*(input.add(offset) as *const Pubkey);
    offset += sizes::OWNER;

    let lamports = {
        let slice = { unsafe { input.add(offset) } };
        offset += sizes::LAMPORTS;
        let lamports_ptr = &mut *(slice as *mut u64);
        Rc::new(RefCell::new(lamports_ptr))
    };

    let data_len = *(input.add(offset) as *const u64) as usize;
    offset += sizes::DATA_LEN_FIELD;

    {
        let data_len_u32 = u32::try_from(data_len).expect("u32::MAX > max account data len");
        *(input.add(original_data_len_offset) as *mut u32) = data_len_u32;
    }

    let data = {
        let slice = core::slice::from_raw_parts_mut(input.add(offset), data_len);
        Rc::new(RefCell::new(slice))
    };

    // ybd:
    // rent_epoch was removed in solana-sdk commit 449d97c0ed
    // here we deserialize as of right before the changes from that commit
    offset += data_len + sizes::MAX_PERMITTED_DATA_INCREASE;
    offset += sizes::padding(offset);

    let rent_epoch = *(input.add(offset) as *const u64);
    offset += sizes::RENT_EPOCH;

    // in the solana-sdk deserialization routine, this offset is returned
    _ = offset;

    AccountInfo {
        key,
        is_signer,
        is_writable,
        lamports,
        data,
        owner,
        executable,
        rent_epoch,
    }
}

pub fn cvlr_deserialize_nondet_accounts_n<'a, const N: usize>() -> [AccountInfo<'a>; N] {
    core::array::from_fn(|_| cvlr_new_account_info())
}

/// for API backwards-compatibility
pub fn cvlr_deserialize_nondet_accounts<'a>() -> [AccountInfo<'a>; 16] {
    cvlr_deserialize_nondet_accounts_n::<16>()
}
