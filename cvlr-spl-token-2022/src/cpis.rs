//! This module provides the functionality for the Certora Prover to
//! automatically mock SPL Token Program instructions.

use cvlr_asserts::*;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
};

/*******************************************************************************
 * Instructions creation
 ******************************************************************************/

/// Creates a `Transfer` instruction.
/// Substitutes https://docs.rs/spl-token/8.0.0/spl_token/instruction/fn.transfer.html.
#[cvlr_early_panic::early_panic]
#[inline(always)]
pub fn transfer(
    token_program_id: &Pubkey,
    source_pubkey: &Pubkey,
    destination_pubkey: &Pubkey,
    authority_pubkey: &Pubkey,
    signer_pubkeys: &[&Pubkey],
    amount: u64,
) -> Result<Instruction, ProgramError> {
    spl_token::check_program_account(token_program_id).unwrap();
    let data = spl_token::instruction::TokenInstruction::Transfer { amount }.pack();

    let mut accounts = Vec::with_capacity(3 + signer_pubkeys.len());
    accounts.push(AccountMeta::new(*source_pubkey, false));
    accounts.push(AccountMeta::new(*destination_pubkey, false));
    accounts.push(AccountMeta::new_readonly(
        *authority_pubkey,
        signer_pubkeys.is_empty(),
    ));
    for signer_pubkey in signer_pubkeys.iter() {
        accounts.push(AccountMeta::new_readonly(**signer_pubkey, true));
    }

    let pubkey = write_spl_token_pubkey();
    Ok(Instruction {
        program_id: pubkey,
        accounts,
        data,
    })
}

/// Creates a `MintTo` instruction.
/// Substitutes https://docs.rs/spl-token/8.0.0/spl_token/instruction/fn.mint_to.html.
#[cvlr_early_panic::early_panic]
pub fn mint_to(
    token_program_id: &Pubkey,
    mint_pubkey: &Pubkey,
    account_pubkey: &Pubkey,
    owner_pubkey: &Pubkey,
    signer_pubkeys: &[&Pubkey],
    amount: u64,
) -> Result<Instruction, ProgramError> {
    spl_token::check_program_account(token_program_id).unwrap();
    let data = spl_token::instruction::TokenInstruction::MintTo { amount }.pack();

    let mut accounts = Vec::with_capacity(3 + signer_pubkeys.len());
    accounts.push(AccountMeta::new(*mint_pubkey, false));
    accounts.push(AccountMeta::new(*account_pubkey, false));
    accounts.push(AccountMeta::new_readonly(
        *owner_pubkey,
        signer_pubkeys.is_empty(),
    ));
    for signer_pubkey in signer_pubkeys.iter() {
        accounts.push(AccountMeta::new_readonly(**signer_pubkey, true));
    }

    let pubkey = write_spl_token_pubkey();
    Ok(Instruction {
        program_id: pubkey,
        accounts,
        data,
    })
}

/// Creates a `Burn` instruction.
/// Substitutes https://docs.rs/spl-token/8.0.0/spl_token/instruction/fn.burn.html.
#[cvlr_early_panic::early_panic]
pub fn burn(
    token_program_id: &Pubkey,
    account_pubkey: &Pubkey,
    mint_pubkey: &Pubkey,
    authority_pubkey: &Pubkey,
    signer_pubkeys: &[&Pubkey],
    amount: u64,
) -> Result<Instruction, ProgramError> {
    spl_token::check_program_account(token_program_id).unwrap();
    let data = spl_token::instruction::TokenInstruction::Burn { amount }.pack();

    let mut accounts = Vec::with_capacity(3 + signer_pubkeys.len());
    accounts.push(AccountMeta::new(*account_pubkey, false));
    accounts.push(AccountMeta::new(*mint_pubkey, false));
    accounts.push(AccountMeta::new_readonly(
        *authority_pubkey,
        signer_pubkeys.is_empty(),
    ));
    for signer_pubkey in signer_pubkeys.iter() {
        accounts.push(AccountMeta::new_readonly(**signer_pubkey, true));
    }

    let pubkey = write_spl_token_pubkey();
    Ok(Instruction {
        program_id: pubkey,
        accounts,
        data,
    })
}

/// Creates a `CloseAccount` instruction.
/// Substitutes https://docs.rs/spl-token/8.0.0/spl_token/instruction/fn.close_account.html
#[cvlr_early_panic::early_panic]
pub fn close_account(
    token_program_id: &Pubkey,
    account_pubkey: &Pubkey,
    destination_pubkey: &Pubkey,
    owner_pubkey: &Pubkey,
    signer_pubkeys: &[&Pubkey],
) -> Result<Instruction, ProgramError> {
    /// Mock for `spl_token::instruction::TokenInstruction::CloseAccount.pack()`.
    fn pack_close_account() -> Vec<u8> {
        let mut buf = Vec::with_capacity(9); // 1 byte for the instruction discriminant and 8 bytes of padding.
        let padding: u64 = 0;
        buf.push(9); // 9 is `CloseAccount` discriminant.

        // We extend the buffer with 8 bytes so that the Certora Prover can
        // correctly recognize the Token instruction.
        buf.extend_from_slice(&padding.to_le_bytes()); // 8 bytes of padding
        buf
    }

    spl_token::check_program_account(token_program_id).unwrap();
    // Currently, we cannot use `pack` directly because the PTA analysis in the
    // Certora Prover is not precise enough to handle the case in which the
    // instruction does not carry any data.
    // let data = spl_token::instruction::TokenInstruction::CloseAccount.pack();
    let data = pack_close_account();

    let mut accounts = Vec::with_capacity(3 + signer_pubkeys.len());
    accounts.push(AccountMeta::new(*account_pubkey, false));
    accounts.push(AccountMeta::new(*destination_pubkey, false));
    accounts.push(AccountMeta::new_readonly(
        *owner_pubkey,
        signer_pubkeys.is_empty(),
    ));
    for signer_pubkey in signer_pubkeys.iter() {
        accounts.push(AccountMeta::new_readonly(**signer_pubkey, true));
    }

    let pubkey = write_spl_token_pubkey();
    Ok(Instruction {
        program_id: pubkey,
        accounts,
        data,
    })
}

/// Creates a `TransferChecked` instruction.
/// Substitutes https://docs.rs/spl-token/8.0.0/spl_token/instruction/fn.transfer_checked.html.
#[allow(clippy::too_many_arguments)]
#[cvlr_early_panic::early_panic]
pub fn transfer_checked(
    token_program_id: &Pubkey,
    source_pubkey: &Pubkey,
    mint_pubkey: &Pubkey,
    destination_pubkey: &Pubkey,
    authority_pubkey: &Pubkey,
    signer_pubkeys: &[&Pubkey],
    amount: u64,
    decimals: u8,
) -> Result<Instruction, ProgramError> {
    spl_token::check_program_account(token_program_id).unwrap();
    let data =
        spl_token::instruction::TokenInstruction::TransferChecked { amount, decimals }.pack();

    let mut accounts = Vec::with_capacity(4 + signer_pubkeys.len());
    accounts.push(AccountMeta::new(*source_pubkey, false));
    accounts.push(AccountMeta::new_readonly(*mint_pubkey, false));
    accounts.push(AccountMeta::new(*destination_pubkey, false));
    accounts.push(AccountMeta::new_readonly(
        *authority_pubkey,
        signer_pubkeys.is_empty(),
    ));
    for signer_pubkey in signer_pubkeys.iter() {
        accounts.push(AccountMeta::new_readonly(**signer_pubkey, true));
    }

    let pubkey = write_spl_token_pubkey();
    Ok(Instruction {
        program_id: pubkey,
        accounts,
        data,
    })
}

/// Writes the Pubkey of the `spl_token::id()` directly into a `Pubkey` and
/// returns it. This is used to ensure that the Certora Prover can
/// recognize the `spl_token` program ID when analyzing the CPI invocations.
#[inline(always)]
fn write_spl_token_pubkey() -> Pubkey {
    #[allow(deprecated)]
    let mut pubkey = Pubkey::default();
    unsafe {
        // Get a mutable pointer to the first byte.
        let ptr = &mut pubkey as *mut Pubkey as *mut u64;
        // Write u64s directly. The following corresponds to the Pubkey of the
        // `spl_token::id()`.
        *ptr.add(0) = 10637895772709248262u64;
        *ptr.add(1) = 12428223917890587609u64;
        *ptr.add(2) = 10463932726783620124u64;
        *ptr.add(3) = 12178014311288245306u64;
    }
    pubkey
}

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
