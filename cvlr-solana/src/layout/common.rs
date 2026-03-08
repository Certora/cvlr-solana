pub mod sizes {
    use core::mem::size_of;
    use solana_program::entrypoint;
    use solana_program::pubkey::Pubkey;

    pub use entrypoint::{BPF_ALIGN_OF_U128, MAX_PERMITTED_DATA_INCREASE};

    pub const NON_DUP_MARKER: usize = size_of::<u8>();
    pub const IS_SIGNER: usize = size_of::<bool>();
    pub const IS_WRITABLE: usize = size_of::<bool>();
    pub const EXECUTABLE: usize = size_of::<bool>();
    pub const ORIGINAL_DATA_LEN: usize = size_of::<u32>();
    pub const KEY: usize = size_of::<Pubkey>();
    pub const OWNER: usize = size_of::<Pubkey>();
    pub const LAMPORTS: usize = size_of::<u64>();
    pub const DATA_LEN_FIELD: usize = size_of::<u64>();
    pub const RENT_EPOCH: usize = size_of::<u64>();

    /// computes the number of padding bytes needed so that the next field
    /// starts at an address aligned to [`BPF_ALIGN_OF_U128`]
    pub fn padding(offset_from_start: usize) -> usize {
        let ptr = offset_from_start as *const u8;
        ptr.align_offset(BPF_ALIGN_OF_U128)
    }

    #[cfg(feature = "rt")]
    /// the "worst-case" amount of padding possible,
    /// which we use to ensure capacity
    pub const fn max_padding() -> usize {
        BPF_ALIGN_OF_U128 - 1
    }
}

pub mod rt_decls {
    extern "C" {
        #[cfg_attr(feature = "rt", allow(dead_code))]
        pub fn CVT_nondet_solana_account_space(size: usize) -> *mut u8;
        pub fn CVT_alloc_slice(base: *mut u8, offset: usize, size: usize) -> *mut u8;
    }
}
