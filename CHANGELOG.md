# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->
## [Unreleased] - ReleaseDate

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
[Unreleased]: https://github.com/Certora/cvlr-solana/compare/cvlr-solana-v0.5.0...HEAD
[0.5.0]: https://github.com/Certora/cvlr-solana/compare/v0.4.5...cvlr-solana-v0.5.0
[0.4.5]: https://github.com/Certora/cvlr-solana/compare/v0.4.3...v0.4.5
[0.4.3]: https://github.com/Certora/cvlr-solana/compare/v0.4.2...v0.4.3
[0.4.2]: https://github.com/Certora/cvlr-solana/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/Certora/cvlr-solana/releases/tag/v0.4.0...v0.4.1
[0.4.0]: https://github.com/Certora/cvlr-solana/releases/tag/v0.3.1...v0.4.0
[0.3.1]: https://github.com/Certora/cvlr-solana/releases/tag/v0.3.1