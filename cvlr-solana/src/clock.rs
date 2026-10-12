use cvlr_nondet::nondet;
use solana_program::clock::Slot;

static mut CVT_CLOCK_SLOT: Option<Slot> = None;

// The prover by default does not inline functions that start with
// prefix "cvt_".  We could tell the prover to inline these functions
// but we tell the rustc to inline them.

/// Advances a modeled clock and returns the new slot.
///
/// Returns an arbitrary slot that is strictly greater than the one returned by
/// the previous call (the first call returns any slot), and records it for
/// [`cvt_get_clock_slot`]. Use these two functions in a mock of the program's
/// clock access (e.g. in place of `Clock::get()?.slot`) to model time moving
/// forward between operations within one rule. They are unrelated to the
/// `Clock` sysvar account; the state lives in a global variable for the
/// duration of the rule.
#[allow(non_snake_case)]
#[inline(always)]
pub fn cvt_get_next_clock_slot() -> Slot {
    unsafe {
        let new_slot = Slot::from(nondet::<u64>());
        if let Some(old_slot) = CVT_CLOCK_SLOT {
            cvlr_asserts::cvlr_assume!(new_slot > old_slot);
        }
        CVT_CLOCK_SLOT = Some(new_slot);
        new_slot
    }
}

/// Returns the slot most recently produced by [`cvt_get_next_clock_slot`].
///
/// Asserts (`cvlr_assert!`) that [`cvt_get_next_clock_slot`] has been called
/// before in the rule; otherwise the rule fails at this call.
#[allow(non_snake_case, static_mut_refs)]
#[inline(always)]
pub fn cvt_get_clock_slot() -> Slot {
    // need to call at least once cvt_get_next_clock_slot before calling this function
    cvlr_asserts::cvlr_assert!(unsafe { CVT_CLOCK_SLOT.is_some() });
    unsafe { CVT_CLOCK_SLOT.unwrap() }
}
