//! This module provides the functionality for the Certora Prover to
//! automatically mock SPL Token Program instructions.
//!
//! # How the substitution works
//!
//! When CPI analysis is enabled (Prover option `-solanaCpiAnalysis true`; it
//! is off by default and experimental), the Solana Prover inspects every call
//! to `solana_program::program::invoke` and `invoke_signed`. If it can resolve
//! the invoked program to SPL Token or Token-2022 and the instruction's first
//! data byte to one of the instructions below, it replaces the call with a
//! call to the matching function of this module:
//!
//! | instruction (discriminant) | `invoke` | `invoke_signed` |
//! | --- | --- | --- |
//! | `Transfer` (3) | [`cvlr_invoke_transfer`] | [`cvlr_invoke_signed_transfer`] |
//! | `MintTo` (7) | [`cvlr_invoke_mint_to`] | [`cvlr_invoke_signed_mint_to`] |
//! | `Burn` (8) | [`cvlr_invoke_burn`] | [`cvlr_invoke_signed_burn`] |
//! | `CloseAccount` (9) | [`cvlr_invoke_close_account`] | [`cvlr_invoke_signed_close_account`] |
//! | `TransferChecked` (12) | [`cvlr_invoke_transfer_checked`] | [`cvlr_invoke_signed_transfer_checked`] |
//!
//! Any other CPI (another program, another instruction, or one that cannot be
//! resolved) is replaced by `assert(false)` so that it cannot be silently
//! ignored: a rule reaching it fails with "Unresolved CPI call". CPIs made
//! through other functions, such as `invoke_signed_unchecked` (which
//! `cvlr_solana::invoke!` uses), are not substituted.
//!
//! The Prover finds these functions by name in the compiled program, so they
//! must be linked into it: invoke
//! [`cvlr_solana_init!`](crate::cvlr_solana_init) once in the program crate.
//!
//! # What the mocks check and model
//!
//! Each mock asserts (`cvlr_assert!`) that the instruction is well formed for
//! the accounts passed: that there are enough account infos (exactly the
//! instruction's account count for the `invoke` forms, at least that many for
//! the `invoke_signed` forms), that the instruction data is not empty, and that
//! the instruction's account metas list the same keys, in the same order, as
//! the account infos. A violation fails the rule. It then reads the `u64`
//! amount from bytes 1..9 of the instruction data and applies the
//! corresponding model from `cvlr_solana::token`, which updates token amounts
//! and supplies without any of the Token program's authority or state checks.
//! Signer seeds are ignored. See those models for exactly what each
//! instruction does.

use cvlr_asserts::*;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    instruction::Instruction,
};
use spl_token::instruction::transfer;

/*******************************************************************************
 * Instructions mocks
 ******************************************************************************/

/// Mock for `Transfer` through `invoke`.
///
/// Asserts that exactly 3 account infos are passed, then behaves as
/// [`cvlr_invoke_signed_transfer`] with no signer seeds.
#[inline(never)]
#[cvlr_early_panic::early_panic]
pub fn cvlr_invoke_transfer(
    instruction: &Instruction,
    account_infos: &[AccountInfo],
) -> ProgramResult {
    cvlr_assert!(account_infos.len() == 3);
    cvlr_invoke_signed_transfer(instruction, account_infos, &[])
}

/// Mock for `Transfer` through `invoke_signed`.
///
/// Accounts, in the order the Token program expects them: source, destination,
/// authority. Asserts that at least 3 account infos are passed, that the
/// instruction data is not empty, and that the first 3 account metas of
/// `instruction` have the same keys as the first 3 account infos; then applies
/// [`cvlr_solana::token::spl_token_transfer`] to those accounts, with the
/// amount from bytes 1..9 of the instruction data. `_signers_seeds` is ignored.
#[inline(never)]
#[cvlr_early_panic::early_panic]
pub fn cvlr_invoke_signed_transfer(
    instruction: &Instruction,
    account_infos: &[AccountInfo],
    _signers_seeds: &[&[&[u8]]],
) -> ProgramResult {
    cvlr_assert!(account_infos.len() >= 3);
    cvlr_assert!(!instruction.data.is_empty());
    cvlr_assert!(instruction.accounts[0].pubkey == *account_infos[0].key);
    cvlr_assert!(instruction.accounts[1].pubkey == *account_infos[1].key);
    cvlr_assert!(instruction.accounts[2].pubkey == *account_infos[2].key);
    let src_info = &account_infos[0];
    let dst_info = &account_infos[1];
    let authority_info = &account_infos[2];
    let amount = u64::from_le_bytes(instruction.data[1..9].try_into().unwrap());
    cvlr_solana::token::spl_token_transfer(src_info, dst_info, authority_info, amount)
}

/// Mock for `MintTo` through `invoke`.
///
/// Asserts that exactly 3 account infos are passed, then behaves as
/// [`cvlr_invoke_signed_mint_to`] with no signer seeds.
#[inline(never)]
#[cvlr_early_panic::early_panic]
pub fn cvlr_invoke_mint_to(
    instruction: &Instruction,
    account_infos: &[AccountInfo],
) -> ProgramResult {
    cvlr_assert!(account_infos.len() == 3);
    cvlr_invoke_signed_mint_to(instruction, account_infos, &[])
}

/// Mock for `MintTo` through `invoke_signed`.
///
/// Accounts, in the order the Token program expects them: mint, destination,
/// authority. Asserts that at least 3 account infos are passed, that the
/// instruction data is not empty, and that the first 3 account metas of
/// `instruction` have the same keys as the first 3 account infos; then applies
/// [`cvlr_solana::token::spl_mint_to`] to those accounts, with the amount from
/// bytes 1..9 of the instruction data. `_signers_seeds` is ignored.
#[inline(never)]
#[cvlr_early_panic::early_panic]
pub fn cvlr_invoke_signed_mint_to(
    instruction: &Instruction,
    account_infos: &[AccountInfo],
    _signers_seeds: &[&[&[u8]]],
) -> ProgramResult {
    cvlr_assert!(account_infos.len() >= 3);
    cvlr_assert!(!instruction.data.is_empty());
    cvlr_assert!(instruction.accounts[0].pubkey == *account_infos[0].key);
    cvlr_assert!(instruction.accounts[1].pubkey == *account_infos[1].key);
    cvlr_assert!(instruction.accounts[2].pubkey == *account_infos[2].key);
    let mint_info = &account_infos[0];
    let dst_info = &account_infos[1];
    let authority_info = &account_infos[2];
    let amount = u64::from_le_bytes(instruction.data[1..9].try_into().unwrap());
    cvlr_solana::token::spl_mint_to(mint_info, dst_info, authority_info, amount)
}

/// Mock for `Burn` through `invoke`.
///
/// Asserts that exactly 3 account infos are passed, then behaves as
/// [`cvlr_invoke_signed_burn`] with no signer seeds.
#[inline(never)]
#[cvlr_early_panic::early_panic]
pub fn cvlr_invoke_burn(instruction: &Instruction, account_infos: &[AccountInfo]) -> ProgramResult {
    cvlr_assert!(account_infos.len() == 3);
    cvlr_invoke_signed_burn(instruction, account_infos, &[])
}

/// Mock for `Burn` through `invoke_signed`.
///
/// Accounts, in the order the Token program expects them: source, mint,
/// authority. Asserts that at least 3 account infos are passed, that the
/// instruction data is not empty, and that the first 3 account metas of
/// `instruction` have the same keys as the first 3 account infos; then applies
/// [`cvlr_solana::token::spl_burn`] to those accounts, with the amount from
/// bytes 1..9 of the instruction data. `_signers_seeds` is ignored.
#[inline(never)]
#[cvlr_early_panic::early_panic]
pub fn cvlr_invoke_signed_burn(
    instruction: &Instruction,
    account_infos: &[AccountInfo],
    _signers_seeds: &[&[&[u8]]],
) -> ProgramResult {
    cvlr_assert!(account_infos.len() >= 3);
    cvlr_assert!(!instruction.data.is_empty());
    cvlr_assert!(instruction.accounts[0].pubkey == *account_infos[0].key);
    cvlr_assert!(instruction.accounts[1].pubkey == *account_infos[1].key);
    cvlr_assert!(instruction.accounts[2].pubkey == *account_infos[2].key);
    let src_info = &account_infos[0];
    let mint_info = &account_infos[1];
    let authority_info = &account_infos[2];
    let amount = u64::from_le_bytes(instruction.data[1..9].try_into().unwrap());
    cvlr_solana::token::spl_burn(mint_info, src_info, authority_info, amount)
}

/// Mock for `CloseAccount` through `invoke`.
///
/// Asserts that exactly 3 account infos are passed, then behaves as
/// [`cvlr_invoke_signed_close_account`] with no signer seeds.
#[inline(never)]
#[cvlr_early_panic::early_panic]
pub fn cvlr_invoke_close_account(
    instruction: &Instruction,
    account_infos: &[AccountInfo],
) -> ProgramResult {
    cvlr_assert!(account_infos.len() == 3);
    cvlr_invoke_signed_close_account(instruction, account_infos, &[])
}

/// Mock for `CloseAccount` through `invoke_signed`.
///
/// Accounts, in the order the Token program expects them: account, destination,
/// owner. Asserts that at least 3 account infos are passed, that the
/// instruction data is not empty, and that the first 3 account metas of
/// `instruction` have the same keys as the first 3 account infos; then applies
/// [`cvlr_solana::token::spl_close_account`] to those accounts.
/// `_signers_seeds` is ignored.
///
/// See the model for how it treats the destination and the source's lamports.
#[inline(never)]
#[cvlr_early_panic::early_panic]
pub fn cvlr_invoke_signed_close_account(
    instruction: &Instruction,
    account_infos: &[AccountInfo],
    _signers_seeds: &[&[&[u8]]],
) -> ProgramResult {
    cvlr_assert!(account_infos.len() >= 3);
    cvlr_assert!(!instruction.data.is_empty());
    cvlr_assert!(instruction.accounts[0].pubkey == *account_infos[0].key);
    cvlr_assert!(instruction.accounts[1].pubkey == *account_infos[1].key);
    cvlr_assert!(instruction.accounts[2].pubkey == *account_infos[2].key);
    let account_info = &account_infos[0];
    let dest_info = &account_infos[1];
    let owner_info = &account_infos[2];
    cvlr_solana::token::spl_close_account(account_info, dest_info, owner_info)
}

/// Mock for `TransferChecked` through `invoke`.
///
/// Asserts that exactly 4 account infos are passed, then behaves as
/// [`cvlr_invoke_signed_transfer_checked`] with no signer seeds.
#[inline(never)]
#[cvlr_early_panic::early_panic]
pub fn cvlr_invoke_transfer_checked(
    instruction: &Instruction,
    account_infos: &[AccountInfo],
) -> ProgramResult {
    cvlr_assert!(account_infos.len() == 4);
    cvlr_invoke_signed_transfer_checked(instruction, account_infos, &[])
}

/// Mock for `TransferChecked` through `invoke_signed`.
///
/// Accounts, in the order the Token program expects them: source, mint,
/// destination, authority. Asserts that at least 4 account infos are passed,
/// that the instruction data is not empty, and that the first 4 account metas
/// of `instruction` have the same keys as the first 4 account infos; then
/// applies [`cvlr_solana::token::spl_token_transfer`] to those accounts, with
/// the amount from bytes 1..9 of the instruction data. `_signers_seeds` is
/// ignored.
///
/// The mint and the `decimals` byte of the instruction are not checked; the
/// model is the same as for `Transfer`.
#[inline(never)]
#[cvlr_early_panic::early_panic]
pub fn cvlr_invoke_signed_transfer_checked(
    instruction: &Instruction,
    account_infos: &[AccountInfo],
    _signers_seeds: &[&[&[u8]]],
) -> ProgramResult {
    cvlr_assert!(account_infos.len() >= 4);
    cvlr_assert!(!instruction.data.is_empty());
    cvlr_assert!(instruction.accounts[0].pubkey == *account_infos[0].key);
    cvlr_assert!(instruction.accounts[1].pubkey == *account_infos[1].key);
    cvlr_assert!(instruction.accounts[2].pubkey == *account_infos[2].key);
    cvlr_assert!(instruction.accounts[3].pubkey == *account_infos[3].key);
    let src_info = &account_infos[0];
    let dst_info = &account_infos[2];
    let authority_info = &account_infos[3];
    let amount = u64::from_le_bytes(instruction.data[1..9].try_into().unwrap());
    cvlr_solana::token::spl_token_transfer(src_info, dst_info, authority_info, amount)
}

/// Macro to initialize the CVLR Solana module.
/// This macro is used to set up the necessary code for the Certora Prover to
/// automatically mock SPL Token Program instructions.
///
/// `cvlr_solana_init!()` defines `#[no_mangle] pub fn init_cvlr_solana()`, and
/// `cvlr_solana_init!(name)` defines it under the given name; the function
/// calls [`make_invoke_mocks_available`]. Invoke it once, at module level, in
/// the crate of the program under verification (typically behind the
/// verification feature). The function exists only so that the CPI mocks are
/// linked into the program for the Prover to find; it is not meant to be
/// called or verified.
#[macro_export]
macro_rules! cvlr_solana_init {
    () => {
        $crate::cvlr_solana_init!(init_cvlr_solana);
    };

    ($wrapper_name:ident) => {
        #[no_mangle]
        pub fn $wrapper_name() {
            $crate::cpis::make_invoke_mocks_available();
        }
    };
}

/// This function is called to make the invoke mocks available for the
/// Certora Prover to use. This can be automatically injected in the analyzed
/// code with the `cvlr_solana_init!` macro.
///
/// It calls every mock of this module once on arbitrary accounts, which keeps
/// them in the compiled program. It has no use in a rule.
pub fn make_invoke_mocks_available() {
    let account_infos = cvlr_solana::cvlr_deserialize_nondet_accounts();
    let account_info_iter = &mut account_infos.iter();
    let acc1: &AccountInfo = next_account_info(account_info_iter).unwrap();
    let acc2: &AccountInfo = next_account_info(account_info_iter).unwrap();
    let acc3: &AccountInfo = next_account_info(account_info_iter).unwrap();
    let acc4: &AccountInfo = next_account_info(account_info_iter).unwrap();
    let nondet: u64 = cvlr_nondet::nondet();
    let mut token_instruction_data = Vec::new();
    token_instruction_data.extend_from_slice(&nondet.to_le_bytes());
    // Any instruction could be used here.
    let instruction = transfer(acc1.key, acc2.key, acc3.key, acc4.key, &[], nondet).unwrap();
    cvlr_invoke_transfer(&instruction, &account_infos).unwrap();
    cvlr_invoke_signed_transfer(&instruction, &account_infos, &[]).unwrap();
    cvlr_invoke_burn(&instruction, &account_infos).unwrap();
    cvlr_invoke_signed_burn(&instruction, &account_infos, &[]).unwrap();
    cvlr_invoke_mint_to(&instruction, &account_infos).unwrap();
    cvlr_invoke_signed_mint_to(&instruction, &account_infos, &[]).unwrap();
    cvlr_invoke_close_account(&instruction, &account_infos).unwrap();
    cvlr_invoke_signed_close_account(&instruction, &account_infos, &[]).unwrap();
    cvlr_invoke_transfer_checked(&instruction, &account_infos).unwrap();
    cvlr_invoke_signed_transfer_checked(&instruction, &account_infos, &[]).unwrap();
}
