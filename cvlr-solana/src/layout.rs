use solana_account_info::AccountInfo;

/// The same as
/// [`cvlr_deserialize_nondet_accounts()`](crate::cvlr_deserialize_nondet_accounts).
///
/// Despite its name, this macro pins no memory layout; it is kept for existing
/// specs.
#[macro_export]
macro_rules! acc_infos_with_mem_layout {
    () => {
        $crate::cvlr_deserialize_nondet_accounts()
    };
}

/// Returns one `AccountInfo` with arbitrary contents, laid out as the Solana
/// runtime lays out program inputs.
///
/// Each call reserves a fresh, fixed memory region for the account (the Prover
/// gives every call its own 10 MiB range in the SVM input region), so the
/// `key`, `owner`, `lamports` and `data` of different accounts never share
/// memory. Their *values* are unconstrained:
///
/// * `key` and `owner` are arbitrary. In particular, two accounts may have the
///   same key; assume `a.key != b.key` where the program relies on distinct
///   accounts.
/// * `is_signer`, `is_writable` and `executable` are arbitrary.
/// * `lamports` and the `_unused` field (formerly `rent_epoch`) are arbitrary
///   `u64`s.
/// * `data` has an arbitrary length of at most 8 MiB and arbitrary contents,
///   with room after it for the runtime's permitted realloc increase
///   (`MAX_PERMITTED_DATA_INCREASE`); the recorded original data length equals
///   the length.
///
/// Constrain the account with `cvlr_assume!` (e.g. on its owner, data length
/// or deserialized contents) to match what the program expects. Usually called
/// through [`cvlr_deserialize_nondet_accounts`].
pub fn cvlr_new_account_info<'a>() -> AccountInfo<'a> {
    unsafe { cvlr_new_account_info_unchecked() }
}

mod rt_decls {
    extern "C" {
        pub fn CVT_nondet_solana_account_space(size: usize) -> *mut u8;
        pub fn CVT_alloc_slice(base: *mut u8, offset: usize, size: usize) -> *mut u8;
    }
}

const BPF_ALIGN_OF_U128: usize = 8;

#[cfg(feature = "rt")]
mod rt_impls {
    use super::BPF_ALIGN_OF_U128;
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

#[allow(unused_assignments)]
unsafe fn cvlr_new_account_info_unchecked<'a>() -> AccountInfo<'a> {
    use cvlr_asserts::cvlr_assume;
    use rt_decls::CVT_alloc_slice;
    use solana_account_info::{AccountInfo, MAX_PERMITTED_DATA_INCREASE};
    use solana_pubkey::Pubkey;
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

/// Returns 16 arbitrary accounts: the standard input of a Solana rule.
///
/// Each element is a separate [`cvlr_new_account_info`], so the accounts
/// occupy disjoint memory but their keys, owners, flags, balances and data are
/// unconstrained and independent; in particular keys may coincide. Pass the
/// array (as a slice) to the program's processor or instruction handler, and
/// assume whatever relationships between accounts the scenario requires:
///
/// ```ignore
/// #[rule]
/// fn rule_withdraw() {
///     let accounts = cvlr_deserialize_nondet_accounts();
///     let (vault, user) = (&accounts[0], &accounts[1]);
///     cvlr_assume!(vault.key != user.key);
///     let amount: u64 = nondet();
///     process_withdraw(&program_id(), &accounts, amount).unwrap();
///     // assertions over vault / user
/// }
/// ```
///
/// Also exported as `cvlr_nondet_acc_infos`.
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
