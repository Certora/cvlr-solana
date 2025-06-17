//! Model fo Stake Program
pub mod utils;
pub mod state;
pub mod tools;
pub mod processor;

// const PERPETUAL_NEW_WARMUP_COOLDOWN_RATE_EPOCH: Option<u64> = Some(0);

#[inline]
pub fn get_minimum_delegation() -> u64 {
    tools::get_minimum_delegation().unwrap()
}