# CVLR for Solana

Solana-specific components of the [CVLR](https://github.com/Certora/cvlr) (Certora Verification Language for Rust) library, enabling formal verification of Solana programs using the Certora Prover.

## Overview

This library provides Solana-specific abstractions and models for formal verification of Solana programs. It includes:

- **Core Solana primitives**: Account info handling, pubkeys, clock, and other Solana runtime abstractions
- **SPL Token Program model**: Formal model of the SPL Token Program for verifying token operations
- **Solana Stake Program model**: Formal model of the Solana Stake Program for verifying staking operations
- **Nondeterministic values**: Support for symbolic execution and formal verification
- **Verification utilities**: Assertions, logging, and other tools for writing formal specifications

## Crates

This workspace contains three crates:

### `cvlr-solana`

Core Solana-specific functionality including:
- Account info handling and deserialization
- Pubkey utilities and logging
- Clock and time abstractions
- Token account utilities
- Nondeterministic value generation for verification

### `cvlr-spl-token`

Formal model of the SPL Token Program, providing:
- Mock implementations of token instructions (transfer, mint, burn, etc.)
- Support for cross-program invocations (CPIs) in formal verification
- Token account state modeling

### `cvlr-solana-stake`

Formal model of the Solana Stake Program, providing:
- Stake account state modeling
- Stake instruction processing
- Support for verifying staking operations

## Usage

Add the relevant crates to your `Cargo.toml`:

```toml
[dependencies]
cvlr-solana = "0.5.0"
cvlr-spl-token = "0.5.0"  # If you need SPL Token modeling
cvlr-solana-stake = "0.5.0"  # If you need Stake Program modeling
```

## Documentation

- [API Documentation](https://docs.rs/cvlr/latest)
- [Certora Website](https://www.certora.com)
- [Repository](https://github.com/Certora/cvlr-solana)

## License

This project is licensed under the MIT License.

## Release

**Current release:** `0.5.0` 
