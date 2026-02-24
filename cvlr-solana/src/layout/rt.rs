use super::common::rt_decls;
use core::cell::RefCell;
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::{BPF_ALIGN_OF_U128, MAX_PERMITTED_DATA_INCREASE};
use solana_program::pubkey::Pubkey;
use std::rc::Rc;

pub(crate) mod global_database {
    use core::ptr;
    use solana_sdk::account::Account;
    use std::sync::{Mutex, OnceLock};

    #[derive(Debug)]
    pub struct GlobalDatabase {
        accounts: Vec<Account>,
    }

    // we assume this is single-threaded.
    pub fn init(accounts: Vec<Account>) {
        let database = GlobalDatabase { accounts };
        let guard = Mutex::new(database);

        // ybd: do we want to allow re-init?
        GLOBAL_DATABASE.set(guard).expect("can only be set once");
    }

    pub static GLOBAL_DATABASE: OnceLock<Mutex<GlobalDatabase>> = OnceLock::new();

    impl GlobalDatabase {
        pub fn get(&self, idx: usize) -> Option<&Account> {
            self.accounts.get(idx)
        }

        pub(crate) fn account_ptr(&mut self, idx: usize) -> Option<ptr::NonNull<u8>> {
            let account = self.accounts.get_mut(idx)?;
            let ptr = ptr::NonNull::from_mut(account).cast();
            Some(ptr)
        }
    }
}

pub fn cvlr_new_account_info<'a>(idx: usize) -> AccountInfo<'a> {
    unsafe { cvlr_new_account_info_rt(idx) }
}

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

pub fn cvlr_deserialize_nondet_accounts<'a, const N: usize>() -> [AccountInfo<'a>; N] {
    core::array::from_fn(cvlr_new_account_info)
}
