//! This module provides the functionality for the Certora Prover to
//! automatically mock SPL Token Program instructions.

use cvlr_asserts::*;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    instruction::Instruction
};
use spl_token::instruction::transfer;

/*******************************************************************************
 * Instructions mocks
 ******************************************************************************/

#[inline(never)]
#[cvlr_early_panic::early_panic]
pub fn cvlr_invoke_transfer(
    instruction: &Instruction,
    account_infos: &[AccountInfo],
) -> ProgramResult {
    cvlr_assert!(account_infos.len() == 3);
    cvlr_invoke_signed_transfer(instruction, account_infos, &[])
}

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

#[inline(never)]
#[cvlr_early_panic::early_panic]
pub fn cvlr_invoke_mint_to(
    instruction: &Instruction,
    account_infos: &[AccountInfo],
) -> ProgramResult {
    cvlr_assert!(account_infos.len() == 3);
    cvlr_invoke_signed_mint_to(instruction, account_infos, &[])
}

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

#[inline(never)]
#[cvlr_early_panic::early_panic]
pub fn cvlr_invoke_burn(instruction: &Instruction, account_infos: &[AccountInfo]) -> ProgramResult {
    cvlr_assert!(account_infos.len() == 3);
    cvlr_invoke_signed_burn(instruction, account_infos, &[])
}

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

#[inline(never)]
#[cvlr_early_panic::early_panic]
pub fn cvlr_invoke_close_account(
    instruction: &Instruction,
    account_infos: &[AccountInfo],
) -> ProgramResult {
    cvlr_assert!(account_infos.len() == 3);
    cvlr_invoke_signed_close_account(instruction, account_infos, &[])
}

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

#[inline(never)]
#[cvlr_early_panic::early_panic]
pub fn cvlr_invoke_transfer_checked(
    instruction: &Instruction,
    account_infos: &[AccountInfo],
) -> ProgramResult {
    cvlr_assert!(account_infos.len() == 4);
    cvlr_invoke_signed_transfer_checked(instruction, account_infos, &[])
}

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
