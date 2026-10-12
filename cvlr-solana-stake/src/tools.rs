//! Tools
//!
//! The minimum delegation, as a `Result` like the real program's helper.

use solana_program::program_error::ProgramError;

/// Returns minimum delegation
///
/// Always `Ok(1)`; see [`crate::get_minimum_delegation`].
pub fn get_minimum_delegation() -> Result<u64, ProgramError> {
    // set to 1. Alternatively, pick some nondet value
    Ok(1)
}
