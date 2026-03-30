use solana_account_info::AccountInfo;

mod rt_decls {
    extern "C" {
        pub fn CVT_nondet_solana_account_space(size: usize) -> *mut u8;
        pub fn CVT_alloc_slice(base: *mut u8, offset: usize, size: usize) -> *mut u8;
    }
}

const BPF_ALIGN_OF_U128: usize = 8;

pub fn cvlr_new_account_info<'a>() -> AccountInfo<'a> {
    unsafe { cvlr_new_account_info_unchecked() }
}



#[cfg(feature = "rt")]
mod rt_impls {
    use super::BPF_ALIGN_OF_U128;
    use std::alloc::{alloc_zeroed, Layout};

    #[no_mangle]
    extern "C" fn CVT_nondet_solana_account_space(size: usize) -> *mut u8 {
        unsafe {
            let layout = Layout::from_size_align_unchecked(size, BPF_ALIGN_OF_U128);
            let input = alloc_zeroed(layout);
            input
        }
    }
    #[no_mangle]
    extern "C" fn CVT_alloc_slice(base: *mut u8, offset: usize, _size: usize) -> *mut u8 {
        unsafe { base.add(offset) }
    }
}

#[allow(unused_assignments)]
unsafe fn cvlr_new_account_info_unchecked<'a>() -> AccountInfo<'a> {
    use cvlr_asserts::cvlr_assume;
    use rt_decls::CVT_alloc_slice;
    use solana_pubkey::Pubkey;
    use solana_account_info::{AccountInfo, MAX_PERMITTED_DATA_INCREASE};
    use std::{alloc::Layout, cell::RefCell, mem::size_of, rc::Rc};

    const MB: usize = 1024 * 1024;
    const MAX_ORIG_DATA_LEN: usize = 8 * MB;
    const SIZE: usize =
        4 + 4 + 32 + 32 + 8 + 8 + MAX_ORIG_DATA_LEN + MAX_PERMITTED_DATA_INCREASE + 8;

    let layout = Layout::from_size_align_unchecked(SIZE, BPF_ALIGN_OF_U128);
    let input: *mut u8 = rt_decls::CVT_nondet_solana_account_space(layout.size());

    let mut offset: usize = 0;

    offset += size_of::<u8>();

    let is_signer = *(input.add(offset) as *const u8) != 0;
    offset += size_of::<u8>();

    let is_writable = *(input.add(offset) as *const u8) != 0;
    offset += size_of::<u8>();

    let executable = *(input.add(offset) as *const u8) != 0;
    offset += size_of::<u8>();

    let len_with_key_ptr: *mut u8 =
        CVT_alloc_slice(input, offset, size_of::<u32>() + size_of::<Pubkey>());

    let original_data_len: u32 = *(len_with_key_ptr as *const u32);
    offset += size_of::<u32>();

    // let key: &Pubkey = &*(input.add(offset) as *const Pubkey);
    let key: &Pubkey = &*(len_with_key_ptr.add(size_of::<u32>()) as *const Pubkey);
    offset += size_of::<Pubkey>();

    // let owner: &Pubkey = &*(input.add(offset) as *const Pubkey);
    let owner: &Pubkey = &*(CVT_alloc_slice(input, offset, size_of::<Pubkey>()) as *const Pubkey);
    offset += size_of::<Pubkey>();

    // let lamports_ptr: &u64 = &mut *(input.add(offset) as *mut u64);
    let lamports_ptr: &mut u64 =
        &mut *(CVT_alloc_slice(input, offset, size_of::<u64>()) as *mut u64);
    cvlr_assume!(cvlr_mathint::is_u64(*lamports_ptr));
    let lamports = Rc::new(RefCell::new(lamports_ptr));
    offset += size_of::<u64>();

    let data_len: usize = cvlr_nondet::nondet::<usize>();
    // -- limit size of data to what is allocated
    cvlr_assume!(data_len <= MAX_ORIG_DATA_LEN);
    // -- ensure that original data len is recorded properly
    cvlr_assume!(original_data_len == data_len as u32);

    let len_with_data_ptr = CVT_alloc_slice(
        input,
        offset,
        size_of::<u64>() + data_len + MAX_PERMITTED_DATA_INCREASE,
    );

    cvlr_assume!(data_len == *(len_with_data_ptr as *const u64) as usize);
    let data_ptr: *mut u8 = len_with_data_ptr.add(size_of::<u64>());
    let data = Rc::new(RefCell::new(std::slice::from_raw_parts_mut(
        data_ptr, data_len,
    )));

    offset += data_len + MAX_PERMITTED_DATA_INCREASE;
    offset += (offset as *const u8).align_offset(BPF_ALIGN_OF_U128);

    // -- place rent_epoch at the end of the data segment
    offset = SIZE - size_of::<u64>();
    let rent_epoch = *(input.add(offset) as *const u64);
    cvlr_assume!(cvlr_mathint::is_u64(rent_epoch));
    offset += size_of::<u64>();

    #[allow(deprecated)]
    AccountInfo {
        key,
        is_signer,
        is_writable,
        lamports,
        data,
        owner,
        executable,
        _unused: rent_epoch,
    }
}


pub fn cvlr_deserialize_nondet_accounts<'a>() -> [AccountInfo<'a>; 16] {
    [
        cvlr_new_account_info(),
        cvlr_new_account_info(),
        cvlr_new_account_info(),
        cvlr_new_account_info(),
        cvlr_new_account_info(),
        cvlr_new_account_info(),
        cvlr_new_account_info(),
        cvlr_new_account_info(),
        cvlr_new_account_info(),
        cvlr_new_account_info(),
        cvlr_new_account_info(),
        cvlr_new_account_info(),
        cvlr_new_account_info(),
        cvlr_new_account_info(),
        cvlr_new_account_info(),
        cvlr_new_account_info(),
    ]
}
