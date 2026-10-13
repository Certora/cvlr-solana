# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->
## [Unreleased] - ReleaseDate
### Changed
  - Built against the Solana SDK 3 split crates (`solana-account-info`,
    `solana-pubkey`, `solana-cpi`, ...) instead of `solana-program` 2.x.
    Programs still on `solana-program` 2.x stay on 0.5.x.
  - `cvlr-spl-token` is built against `spl-token` 9.
  - `cvlr-solana-stake` takes its stake types from `solana-stake-interface` 5.
  - `invoke!` and `invoke_signed!` call `solana_cpi::invoke_signed_unchecked`,
    and no longer need the calling crate to depend on `solana-program`.
  - The `rt` feature uses `solana-account` instead of `solana-sdk`.
### Removed
  - `fun_acc_infos_with_mem_layout`. Use `cvlr_deserialize_nondet_accounts`;
    `acc_infos_with_mem_layout!` remains as an alias for it.

## [0.5.1] - 2026-10-13
### Added
  - Documentation for every public item of `cvlr-solana`, `cvlr-spl-token` and
    `cvlr-solana-stake`; CI now fails on undocumented public items.
### Fixed
  - The `rt` feature builds and links: `solana-sdk` is an optional dependency
    it enables, it forwards `rt` to `cvlr-log`, and `cvlr-asserts` is required
    at 0.4.2 or later.

## [0.5.0] - 2026-01-16
### Added
  - model of SPL Stake in cvlr-solana-stake
### Changed
  - model of SPL Token is factored out into cvlr-spl-token crate
  - bump solana program dependency to 2.x

## [0.4.5] 2026-01-16

### Fixed
  - summary of `spl_burn`
  - Hide static lifetime in `cvlr_new_account_info_unchecked`

## [0.4.3] 2025-04-09

### Added

### Changed
  - Update spl-token and spl-token-2022 version requirements to match

### Removed
  - yank version v0.4.2 because of too many issues with dependency      resolution

## [0.4.2] 2025-04-09

### Added

### Changed
  - Restrict Solana version to v1.18

### Removed
  - yank version v0.4.1 because of too many issues with dependency      resolution


## [0.4.1] 2025-04-08

### Added
  - `pubkey::Pk` is wrapper for Pubkey for easy logging 

### Changed

### Removed

## [0.4.0] - 2025-04-04

### Added
  - prepare release on crates.io
### Changed

### Removed


## [0.3.1] - 2025-04-04

### Added
  - This is the first official release

### Fixed

### Changed

### Removed

<!-- next-url -->
[Unreleased]: https://github.com/Certora/cvlr-solana/compare/cvlr-solana-v0.5.1...HEAD
[0.5.1]: https://github.com/Certora/cvlr-solana/compare/cvlr-solana-v0.5.0...cvlr-solana-v0.5.1
[0.5.0]: https://github.com/Certora/cvlr-solana/compare/v0.4.5...cvlr-solana-v0.5.0
[0.4.5]: https://github.com/Certora/cvlr-solana/compare/v0.4.3...v0.4.5
[0.4.3]: https://github.com/Certora/cvlr-solana/compare/v0.4.2...v0.4.3
[0.4.2]: https://github.com/Certora/cvlr-solana/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/Certora/cvlr-solana/releases/tag/v0.4.0...v0.4.1
[0.4.0]: https://github.com/Certora/cvlr-solana/releases/tag/v0.3.1...v0.4.0
[0.3.1]: https://github.com/Certora/cvlr-solana/releases/tag/v0.3.1