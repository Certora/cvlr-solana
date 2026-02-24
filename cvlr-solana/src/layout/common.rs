pub mod rt_decls {
    use core::ptr;

    extern "C" {
        #[cfg_attr(feature = "rt", allow(dead_code))]
        pub fn CVT_nondet_solana_account_space(size: usize) -> *mut u8;
        #[cfg_attr(not(feature = "rt"), allow(dead_code))]
        pub fn CVT_deserialize_global_account(idx: usize) -> Option<ptr::NonNull<u8>>;
        pub fn CVT_alloc_slice(base: *mut u8, offset: usize, size: usize) -> *mut u8;
    }
}
