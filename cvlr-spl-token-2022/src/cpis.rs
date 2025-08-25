//! This module provides the functionality for the Certora Prover to
//! automatically mock SPL Token Program instructions.

use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
};
use spl_token_2022::{check_spl_token_program_account, instruction::TokenInstruction};

/*******************************************************************************
 * Instructions creation
 ******************************************************************************/

/// Creates a `TransferChecked` instruction.
/// Substitutes https://docs.rs/spl-token-2022/7.0.0/spl_token_2022/instruction/fn.transfer_checked.html
#[cvlr_early_panic::early_panic]
#[allow(clippy::too_many_arguments)]
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
    check_spl_token_program_account(token_program_id)?;
    let data = TokenInstruction::TransferChecked { amount, decimals }.pack();

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
    Ok(Instruction {
        program_id: write_spl_token_2022_pubkey(),
        accounts,
        data,
    })
}

/// Writes the Pubkey of the `spl_token::id()` directly into a `Pubkey` and
/// returns it. This is used to ensure that the Certora Prover can
/// recognize the `spl_token` program ID when analyzing the CPI invocations.
#[inline(always)]
fn write_spl_token_2022_pubkey() -> Pubkey {
    #[allow(deprecated)]
    let mut pubkey = Pubkey::default();
    unsafe {
        // Get a mutable pointer to the first byte.
        let ptr = &mut pubkey as *mut Pubkey as *mut u64;
        // Write u64s directly. The following corresponds to the Pubkey of the
        // `spl_token_2022::id()`.
        *ptr.add(0) = 16037166466943343878u64;
        *ptr.add(1) = 15766377600162546200u64;
        *ptr.add(2) = 2814109315776649910u64;
        *ptr.add(3) = 18197816669093084670u64;
    }
    pubkey
}