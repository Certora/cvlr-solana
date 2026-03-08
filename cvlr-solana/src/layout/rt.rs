use crate::layout::common::rt_decls;
use crate::layout::common::sizes;
use crate::layout::instruction_accounts::InstructionAccounts;
use core::cell::RefCell;
use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;
use std::rc::Rc;

/// fetches the next unallocated account and reads it to [`AccountInfo`]
pub fn cvlr_new_account_info<'a>() -> AccountInfo<'a> {
    let ptr = InstructionAccounts::next_ptr().expect("next account has not been allocated yet");
    unsafe { cvlr_new_account_info_rt(ptr) }
}

mod rt_impls {
    use solana_program::entrypoint::BPF_ALIGN_OF_U128;
    use std::alloc::{alloc_zeroed, Layout};

    #[no_mangle]
    extern "C" fn CVT_nondet_solana_account_space(size: usize) -> *mut u8 {
        unsafe {
            let layout = Layout::from_size_align_unchecked(size, BPF_ALIGN_OF_U128);
            alloc_zeroed(layout)
        }
    }

    #[no_mangle]
    extern "C" fn CVT_alloc_slice(base: *mut u8, offset: usize, _size: usize) -> *mut u8 {
        unsafe { base.add(offset) }
    }
}

unsafe fn cvlr_new_account_info_rt<'a>(input: *mut u8) -> AccountInfo<'a> {
    use rt_decls::CVT_alloc_slice;
    use solana_program::entrypoint::NON_DUP_MARKER;

    let mut offset: usize = 0;

    let dup_marker = *(input.add(offset) as *const u8);
    if dup_marker == NON_DUP_MARKER {
        offset += sizes::NON_DUP_MARKER;
    } else {
        panic!("acccount detected as duplicate")
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

    let key = {
        let slice = CVT_alloc_slice(input, offset, sizes::KEY);
        offset += sizes::KEY;
        &*(slice as *const Pubkey)
    };

    let owner = {
        let slice = CVT_alloc_slice(input, offset, sizes::OWNER);
        offset += sizes::OWNER;
        &*(slice as *const Pubkey)
    };

    // ybd: it is not clear to me how it's sound to
    // to deserialize Rc<RefCell<T>> as *mut u64,
    // other thah both types having the same size
    let lamports = {
        let slice = CVT_alloc_slice(input, offset, sizes::LAMPORTS);
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
        let slice = CVT_alloc_slice(input, offset, data_len);
        let slice = core::slice::from_raw_parts_mut(slice, data_len);
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
