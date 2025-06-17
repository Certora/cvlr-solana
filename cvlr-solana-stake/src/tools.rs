//! Tools

use solana_program::program_error::ProgramError;

/// Returns minimum delegation
pub fn get_minimum_delegation() -> Result<u64, ProgramError>  {
    // set to 1. Alternatively, pick some nondet value
    Ok(1)
}