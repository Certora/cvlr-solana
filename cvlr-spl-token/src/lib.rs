//! Main library
//!
//! This module provides a wrapper around SPL Token that selectively overrides
//! specific instruction functions with custom implementations while maintaining
//! full backward compatibility with the standard SPL Token API.

// Expose token utilities from cvlr-solana if additional mocking is needed.
pub use cvlr_solana::token::*;

// Contains our custom Cross-Program Invocation (CPI) implementations.
// These are the actual custom logic for overridden SPL Token functions.
pub mod cpis;

// Private base module that re-exports all SPL Token functionality.
// This is kept private to prevent direct access to the original implementations
// of functions we want to override (like transfer, mint_to, etc.)
mod base {
    pub use spl_token::*;
}

// Create a public `instruction` module that overrides specific functions.
// This module provides the public API and selectively exposes our custom
// implementations instead of the original SPL Token versions for certain
// operations.
pub mod instruction {
    // Re-export everything from `spl_token::instruction`.
    pub use spl_token::instruction::*;
}

// Re-export everything from `spl_token` at the top level, except the instruction
// overrides. This makes it possible to just import `cvlr-spl-token` as
// `spl_token` and use it as a drop-in replacement for the original SPL Token
// library. Under the hood, it will use our custom implementations for the
// specified functions while keeping the rest of the SPL Token API intact.
// The key insight: since we re-export from `base` (which contains spl_token::*),
// but we've already defined a public `instruction` module above,
// Rust will use our custom `instruction` module instead of
// `spl_token::instruction` This gives us selective overriding: custom
// implementations for some functions, original implementations for everything
// else.
pub use base::*;
