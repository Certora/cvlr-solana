use crate::nondet::cvlr_nondet_account_info;
use core::cell::RefCell;
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::{BPF_ALIGN_OF_U128, MAX_PERMITTED_DATA_INCREASE};
use solana_program::pubkey::Pubkey;
use std::rc::Rc;

/// Memory layout of AccountInfo field `data` as `Rc<RefCell<&[u8]>>`
macro_rules! mem_layout_rc_data {
    ($acc_info: expr, $start_addr: expr, $num_acc: expr) => {{
        // We need the start of the Rc. The method as_ptr() returns the address of &[u8]
        let data_rc = unsafe { $acc_info.data.as_ptr().offset(-24) } as *const _;
        cvlr_asserts::cvlr_assume!(data_rc == ($start_addr + (512 * $num_acc)) as *const _);
    }};
}

/// Memory layout of AccountInfo field `data` as `[u8]`
macro_rules! mem_layout_data {
    ($acc_info_prev: expr, $acc_info: expr, $data_sz: expr) => {{
        let prev_data_ptr = $acc_info_prev.data.borrow().as_ptr();
        let data_ptr = $acc_info.data.borrow().as_ptr();
        cvlr_asserts::cvlr_assume!((prev_data_ptr as usize + $data_sz) as *const u8 == data_ptr);
    }};
}

/// Memory layout of AccountInfo field `lamports` as `Rc<RefCell<&u64>>`
macro_rules! mem_layout_rc_lamport {
    ($acc_info: expr, $start_addr: expr, $num_acc: expr) => {{
        // We need the start of the Rc. The method as_ptr() returns the address of T
        let lamports_rc = unsafe { $acc_info.lamports.as_ptr().offset(-24) } as *const _;
        cvlr_asserts::cvlr_assume!(lamports_rc == ($start_addr + (64 * $num_acc)) as *const _);
    }};
}

/// Memory layout of AccountInfo field `lamports` as `&u64`
macro_rules! mem_layout_lamport {
    ($acc_info_prev: expr, $acc_info: expr, $data_sz: expr) => {{
        let prev_lamports_ptr = *$acc_info_prev.lamports.borrow() as *const u64;
        let lamports_ptr = *$acc_info.lamports.borrow() as *const u64;
        cvlr_asserts::cvlr_assume!(
            (prev_lamports_ptr as usize + $data_sz) as *const u64 == lamports_ptr
        );
    }};
}

/// Memory layout of AccountInfo field `key` as `&Pubkey`
macro_rules! mem_layout_key {
    ($acc_info: expr, $start_addr: expr, $num_acc: expr) => {{
        let key_ptr = &*$acc_info.key as *const _;
        cvlr_asserts::cvlr_assume!(key_ptr as usize == $start_addr + (64 * $num_acc));
    }};
}

/// Memory layout of AccountInfo field `owner` as `&Pubkey`
macro_rules! mem_layout_owner {
    ($acc_info: expr, $start_addr: expr, $num_acc: expr) => {{
        let owner_ptr = &*$acc_info.owner as *const _;
        cvlr_asserts::cvlr_assume!(owner_ptr as usize == $start_addr + (64 * $num_acc));
    }};
}

/// The function `fun_acc_infos_with_mem_layout` returns 16 AccountInfo
/// initialized non-deterministically.
///
/// While the contents of the accounts are unconstrained, this function
/// assigns fixed addresses to any field that contains a pointer or
/// reference: `key`, `lamports`, `data`, and `owner`.
///
/// The purpose of this is to eliminate spurious counterexamples while
/// making easier the debugging of counterexamples.  This assignment of
/// fixed addresses should be sound since no Solana contract should branch
/// on whether, for instance, a public key starts at some particular
/// address in the SVM (Solana Virtual Machine).
///
/// ```
/// use std::cell::RefCell;
/// use std::rc::Rc;
/// use solana_program::{pubkey::Pubkey, clock::Epoch};
///
/// pub struct AccountInfo<'a> {
///     /// Public key of the account
///     pub key: &'a Pubkey,
///     /// The lamports in the account.  Modifiable by programs.
///     pub lamports: Rc<RefCell<&'a mut u64>>,
///     /// The data held in this account.  Modifiable by programs.
///     pub data: Rc<RefCell<&'a mut [u8]>>,
///     /// Program that owns this account
///     pub owner: &'a Pubkey,
///     /// The epoch at which this account will next owe rent
///     pub rent_epoch: Epoch,
///     /// Was the transaction signed by this account's public key?
///     pub is_signer: bool,
///     /// Is the account writable?
///     pub is_writable: bool,
///     /// This account's data contains a loaded program (and is now read-only)
///     pub executable: bool
/// }
/// ```
pub fn fun_acc_infos_with_mem_layout() -> [AccountInfo<'static>; 16] {
    let acc1 = cvlr_nondet_account_info();
    let acc2 = cvlr_nondet_account_info();
    let acc3 = cvlr_nondet_account_info();
    let acc4 = cvlr_nondet_account_info();
    let acc5 = cvlr_nondet_account_info();
    let acc6 = cvlr_nondet_account_info();
    let acc7 = cvlr_nondet_account_info();
    let acc8 = cvlr_nondet_account_info();
    let acc9 = cvlr_nondet_account_info();
    let acc10 = cvlr_nondet_account_info();
    let acc11 = cvlr_nondet_account_info();
    let acc12 = cvlr_nondet_account_info();
    let acc13 = cvlr_nondet_account_info();
    let acc14 = cvlr_nondet_account_info();
    let acc15 = cvlr_nondet_account_info();
    let acc16 = cvlr_nondet_account_info();

    /*
     *   When the Solana program entrypoint is called the contents of
     *   the AccountInfo are lay out in the "context" (aka "input")
     *   memory region like this:
     *
     *           | AccountInfo 1 | AccountInfo 2 | .... | AccountInfo N |
     *
     *   For convenience, we arrange them differently. First all the data fields, then all the lamport fields, and so on.
     *
     *
     *               16                               16                                   16                 16                 16              16
     *   data &[u8] ... data &[u8] | lamports Rc  ... lamports Rc | lamports &u64 ... lamports &u64 | data Rc ... data Rc  | key ... key   | owner ... owner
     *    ^                           ^                              ^                                 ^                      ^               ^
     *   0x400_000_008             | 0x40A_000_080                | 0x40A_000_480                   | 0x40A_000_500        | 0x40A_002_500 | 0x40A_002_900
     *
     *   <----  16*0xA00_008 -----><----------  16*64=0x400 ------><------------ 16*8=0x80----------><----16*512=0x2000----><-----0x400---><-----0x400----->
     *
     **/
    {
        // layout of data &[u8]
        // The actual address is 0x400_000_000 and it's the start of the context memory region in SVM
        let start_addr: u64 = 0x400_000_008;
        // each account has size of 10MB: 10485760  (0xA00_000). We add 8 just to be conservative.
        let data_sz: usize = 10485760 + 8;

        let acc1_data_ptr = acc1.data.borrow().as_ptr();
        cvlr_asserts::cvlr_assume!(acc1_data_ptr == start_addr as *const u8);
        mem_layout_data!(acc1, acc2, data_sz);
        mem_layout_data!(acc2, acc3, data_sz);
        mem_layout_data!(acc3, acc4, data_sz);
        mem_layout_data!(acc4, acc5, data_sz);
        mem_layout_data!(acc5, acc6, data_sz);
        mem_layout_data!(acc6, acc7, data_sz);
        mem_layout_data!(acc7, acc8, data_sz);
        mem_layout_data!(acc8, acc9, data_sz);
        mem_layout_data!(acc9, acc10, data_sz);
        mem_layout_data!(acc10, acc11, data_sz);
        mem_layout_data!(acc11, acc12, data_sz);
        mem_layout_data!(acc12, acc13, data_sz);
        mem_layout_data!(acc13, acc14, data_sz);
        mem_layout_data!(acc14, acc15, data_sz);
        mem_layout_data!(acc15, acc16, data_sz);
    }
    {
        // layout of lamports Rc<RefCell<T>>
        let start_addr: usize = 0x40A_000_080;
        mem_layout_rc_lamport!(acc1, start_addr, 0);
        mem_layout_rc_lamport!(acc2, start_addr, 1);
        mem_layout_rc_lamport!(acc3, start_addr, 2);
        mem_layout_rc_lamport!(acc4, start_addr, 3);
        mem_layout_rc_lamport!(acc5, start_addr, 4);
        mem_layout_rc_lamport!(acc6, start_addr, 5);
        mem_layout_rc_lamport!(acc7, start_addr, 6);
        mem_layout_rc_lamport!(acc8, start_addr, 7);
        mem_layout_rc_lamport!(acc9, start_addr, 8);
        mem_layout_rc_lamport!(acc10, start_addr, 9);
        mem_layout_rc_lamport!(acc11, start_addr, 10);
        mem_layout_rc_lamport!(acc12, start_addr, 11);
        mem_layout_rc_lamport!(acc13, start_addr, 12);
        mem_layout_rc_lamport!(acc14, start_addr, 13);
        mem_layout_rc_lamport!(acc15, start_addr, 14);
        mem_layout_rc_lamport!(acc16, start_addr, 15);
    }
    {
        // layout of lamports
        let start_addr: usize = 0x40A_000_480;
        let acc1_lamports_ptr = *acc1.lamports.borrow() as *const u64;
        cvlr_asserts::cvlr_assume!(acc1_lamports_ptr == start_addr as *const u64);

        mem_layout_lamport!(acc1, acc2, 8);
        mem_layout_lamport!(acc2, acc3, 8);
        mem_layout_lamport!(acc3, acc4, 8);
        mem_layout_lamport!(acc4, acc5, 8);
        mem_layout_lamport!(acc5, acc6, 8);
        mem_layout_lamport!(acc6, acc7, 8);
        mem_layout_lamport!(acc7, acc8, 8);
        mem_layout_lamport!(acc8, acc9, 8);
        mem_layout_lamport!(acc9, acc10, 8);
        mem_layout_lamport!(acc10, acc11, 8);
        mem_layout_lamport!(acc11, acc12, 8);
        mem_layout_lamport!(acc12, acc13, 8);
        mem_layout_lamport!(acc13, acc14, 8);
        mem_layout_lamport!(acc14, acc15, 8);
        mem_layout_lamport!(acc15, acc16, 8);
    }
    {
        // layout of data Rc<RefCell<...>>
        let start_addr: usize = 0x40A_000_500;
        mem_layout_rc_data!(acc1, start_addr, 0);
        mem_layout_rc_data!(acc2, start_addr, 1);
        mem_layout_rc_data!(acc3, start_addr, 2);
        mem_layout_rc_data!(acc4, start_addr, 3);
        mem_layout_rc_data!(acc5, start_addr, 4);
        mem_layout_rc_data!(acc6, start_addr, 5);
        mem_layout_rc_data!(acc7, start_addr, 6);
        mem_layout_rc_data!(acc8, start_addr, 7);
        mem_layout_rc_data!(acc9, start_addr, 8);
        mem_layout_rc_data!(acc10, start_addr, 9);
        mem_layout_rc_data!(acc11, start_addr, 10);
        mem_layout_rc_data!(acc12, start_addr, 11);
        mem_layout_rc_data!(acc13, start_addr, 12);
        mem_layout_rc_data!(acc14, start_addr, 13);
        mem_layout_rc_data!(acc15, start_addr, 14);
        mem_layout_rc_data!(acc16, start_addr, 15);
    }
    {
        // layout of key
        let start_addr: usize = 0x40A_002_500;
        mem_layout_key!(acc1, start_addr, 0);
        mem_layout_key!(acc2, start_addr, 1);
        mem_layout_key!(acc3, start_addr, 2);
        mem_layout_key!(acc4, start_addr, 3);
        mem_layout_key!(acc5, start_addr, 4);
        mem_layout_key!(acc6, start_addr, 5);
        mem_layout_key!(acc7, start_addr, 6);
        mem_layout_key!(acc8, start_addr, 7);
        mem_layout_key!(acc9, start_addr, 8);
        mem_layout_key!(acc10, start_addr, 9);
        mem_layout_key!(acc11, start_addr, 10);
        mem_layout_key!(acc12, start_addr, 11);
        mem_layout_key!(acc13, start_addr, 12);
        mem_layout_key!(acc14, start_addr, 13);
        mem_layout_key!(acc15, start_addr, 14);
        mem_layout_key!(acc16, start_addr, 15);
    }
    {
        // layout of owner
        let start_addr: usize = 0x40A_002_900;
        mem_layout_owner!(acc1, start_addr, 0);
        mem_layout_owner!(acc2, start_addr, 1);
        mem_layout_owner!(acc3, start_addr, 2);
        mem_layout_owner!(acc4, start_addr, 3);
        mem_layout_owner!(acc5, start_addr, 4);
        mem_layout_owner!(acc6, start_addr, 5);
        mem_layout_owner!(acc7, start_addr, 6);
        mem_layout_owner!(acc8, start_addr, 7);
        mem_layout_owner!(acc9, start_addr, 8);
        mem_layout_owner!(acc10, start_addr, 9);
        mem_layout_owner!(acc11, start_addr, 10);
        mem_layout_owner!(acc12, start_addr, 11);
        mem_layout_owner!(acc13, start_addr, 12);
        mem_layout_owner!(acc14, start_addr, 13);
        mem_layout_owner!(acc15, start_addr, 14);
        mem_layout_owner!(acc16, start_addr, 15);
    }

    [
        acc1, acc2, acc3, acc4, acc5, acc6, acc7, acc8, acc9, acc10, acc11, acc12, acc13, acc14,
        acc15, acc16,
    ]
}

#[macro_export]
macro_rules! acc_infos_with_mem_layout {
    () => {
        $crate::cvlr_deserialize_nondet_accounts()
    };
}

#[cfg(not(feature = "rt"))]
pub fn cvlr_new_account_info<'a>() -> AccountInfo<'a> {
    unsafe { cvlr_new_account_info_unchecked() }
}

#[cfg(feature = "rt")]
pub fn cvlr_new_account_info<'a>(idx: usize) -> AccountInfo<'a> {
    unsafe { cvlr_new_account_info_rt(idx) }
}

mod rt_decls {
    use core::ptr;

    extern "C" {
        #[cfg_attr(feature = "rt", allow(dead_code))]
        pub fn CVT_nondet_solana_account_space(size: usize) -> *mut u8;
        #[cfg_attr(not(feature = "rt"), allow(dead_code))]
        pub fn CVT_deserialize_global_account(idx: usize) -> Option<ptr::NonNull<u8>>;
        pub fn CVT_alloc_slice(base: *mut u8, offset: usize, size: usize) -> *mut u8;
    }
}

#[cfg(feature = "rt")]
mod rt_impls {
    use super::global_database::GLOBAL_DATABASE;
    use core::ptr;
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
    extern "C" fn CVT_deserialize_global_account(idx: usize) -> Option<ptr::NonNull<u8>> {
        // ybd: note that all calls here return Option instead of panicing,
        // because panic is UB here and I'd rather have the caller assert this
        let guard = GLOBAL_DATABASE.get()?;
        let mut db = guard.try_lock().ok()?;
        db.account_ptr(idx)
    }

    #[no_mangle]
    extern "C" fn CVT_alloc_slice(base: *mut u8, offset: usize, _size: usize) -> *mut u8 {
        unsafe { base.add(offset) }
    }
}

#[cfg(feature = "rt")]
pub mod global_database;

#[cfg(feature = "rt")]
unsafe fn cvlr_new_account_info_rt<'a>(idx: usize) -> AccountInfo<'a> {
    use rt_decls::CVT_alloc_slice;

    // copied here to avoid adding a dependency on solana_sdk
    const NON_DUP_MARKER: u8 = u8::MAX;

    let input = rt_decls::CVT_deserialize_global_account(idx)
        .unwrap()
        .as_ptr();

    let mut offset: usize = 0;

    // we don't care about this marker.
    offset += size_of_val(&NON_DUP_MARKER);

    let is_signer = *(input.add(offset) as *const u8) != 0;
    offset += size_of::<u8>();

    let is_writable = *(input.add(offset) as *const u8) != 0;
    offset += size_of::<u8>();

    // ybd: for whatever reason, `executable` is here,
    // and not at the end like in entrypoint_deprecated
    let executable = *(input.add(offset) as *const u8) != 0;
    offset += size_of::<u8>();

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
    offset += size_of::<u32>();

    let key = {
        let slice = CVT_alloc_slice(input, offset, size_of::<Pubkey>());
        offset += size_of::<Pubkey>();
        &*(slice as *const Pubkey)
    };

    let owner = {
        let slice = CVT_alloc_slice(input, offset, size_of::<Pubkey>());
        offset += size_of::<Pubkey>();
        &*(slice as *const Pubkey)
    };

    // ybd: it is not clear to me how it's sound to
    // to deserialize Rc<RefCell<T>> as *mut u64,
    // other thah both types having the same size
    let rc_refcell_size = size_of::<u64>();

    let lamports = {
        let slice = CVT_alloc_slice(input, offset, rc_refcell_size);
        offset += rc_refcell_size;
        let lamports_ptr = &mut *(slice as *mut u64);
        Rc::new(RefCell::new(lamports_ptr))
    };

    let data_len = *(input.add(offset) as *const u64) as usize;

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
    // 1. rent_epoch was removed in solana-sdk commit 449d97c0ed
    // here we deserialize as of right before the changes from that commit
    // 2. here we assume data increase is bounded, as define by solana abi
    offset += data_len + MAX_PERMITTED_DATA_INCREASE;
    offset += (offset as *const u8).align_offset(BPF_ALIGN_OF_U128);

    let rent_epoch = *(input.add(offset) as *const u64);
    offset += size_of::<u64>();

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

#[cfg(not(feature = "rt"))]
#[allow(unused_assignments)]
unsafe fn cvlr_new_account_info_unchecked<'a>() -> AccountInfo<'a> {
    use cvlr_asserts::cvlr_assume;
    use rt_decls::CVT_alloc_slice;
    use std::alloc::Layout;

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

#[cfg(not(feature = "rt"))]
#[allow(unused_assignments)]
unsafe fn _cvlr_new_account_info_unchecked() -> AccountInfo<'static> {
    use solana_program::{
        entrypoint::{BPF_ALIGN_OF_U128, MAX_PERMITTED_DATA_INCREASE},
        pubkey::Pubkey,
    };
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

    let original_data_len_offset = offset;
    offset += size_of::<u32>();

    let key: &Pubkey = &*(input.add(offset) as *const Pubkey);
    offset += size_of::<Pubkey>();

    let owner: &Pubkey = &*(input.add(offset) as *const Pubkey);
    offset += size_of::<Pubkey>();

    let lamports = Rc::new(RefCell::new(&mut *(input.add(offset) as *mut u64)));
    offset += size_of::<u64>();

    let data_len = *(input.add(offset) as *const u64) as usize;
    offset += size_of::<u64>();

    *(input.add(original_data_len_offset) as *mut u32) = data_len as u32;

    // -- limit size of data to what is allocated
    cvlr_asserts::cvlr_assume!(data_len <= MAX_ORIG_DATA_LEN);

    let data = Rc::new(RefCell::new(std::slice::from_raw_parts_mut(
        input.add(offset),
        data_len,
    )));

    offset += data_len + MAX_PERMITTED_DATA_INCREASE;
    offset += (offset as *const u8).align_offset(BPF_ALIGN_OF_U128);

    // let rent_epoch = *(input.add(offset) as *const u64);
    let rent_epoch = cvlr_nondet::nondet::<u64>();
    offset += size_of::<u64>();

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

#[cfg(not(feature = "rt"))]
pub fn cvlr_deserialize_nondet_accounts<'a, const N: usize>() -> [AccountInfo<'a>; N] {
    core::array::from_fn(|_| cvlr_new_account_info())
}

#[cfg(feature = "rt")]
pub fn cvlr_deserialize_nondet_accounts<'a, const N: usize>() -> [AccountInfo<'a>; N] {
    core::array::from_fn(cvlr_new_account_info)
}
