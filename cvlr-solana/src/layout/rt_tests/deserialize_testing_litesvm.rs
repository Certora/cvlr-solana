use crate::layout::rt::instruction_accounts::{AccountData, AccountMeta};
use crate::{cvlr_new_account_info, InstructionAccounts, InstructionAccountsBuilder};
use litesvm::LiteSVM;
use solana_sdk::{
    program_pack::Pack,
    rent::Rent,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::{Transaction, VersionedTransaction},
};
use spl_token::{instruction as token_ix, state as token_state};

#[test]
fn deserialize_testing_litesvm() {
    let mut svm = LiteSVM::new().with_spl_programs();

    let payer = Keypair::new();
    let mint_keypair = Keypair::new();
    let mint_authority = Keypair::new();
    let token_owner = Keypair::new();
    let token_keypair = Keypair::new();

    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    let rent = Rent::default();
    let mint_lamports = rent.minimum_balance(token_state::Mint::LEN);
    let token_lamports = rent.minimum_balance(token_state::Account::LEN);

    let create_mint_tx = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &payer.pubkey(),
                &mint_keypair.pubkey(),
                mint_lamports,
                token_state::Mint::LEN as u64,
                &spl_token::id(),
            ),
            token_ix::initialize_mint2(
                &spl_token::id(),
                &mint_keypair.pubkey(),
                &mint_authority.pubkey(),
                None,
                6,
            )
            .unwrap(),
        ],
        Some(&payer.pubkey()),
        &[&payer, &mint_keypair],
        svm.latest_blockhash(),
    );
    svm.send_transaction(VersionedTransaction::from(create_mint_tx))
        .unwrap();

    let create_token_tx = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &payer.pubkey(),
                &token_keypair.pubkey(),
                token_lamports,
                token_state::Account::LEN as u64,
                &spl_token::id(),
            ),
            token_ix::initialize_account3(
                &spl_token::id(),
                &token_keypair.pubkey(),
                &mint_keypair.pubkey(),
                &token_owner.pubkey(),
            )
            .unwrap(),
        ],
        Some(&payer.pubkey()),
        &[&payer, &token_keypair],
        svm.latest_blockhash(),
    );
    svm.send_transaction(VersionedTransaction::from(create_token_tx))
        .unwrap();

    let mint_to_tx = Transaction::new_signed_with_payer(
        &[token_ix::mint_to(
            &spl_token::id(),
            &mint_keypair.pubkey(),
            &token_keypair.pubkey(),
            &mint_authority.pubkey(),
            &[],
            1_000_000,
        )
        .unwrap()],
        Some(&payer.pubkey()),
        &[&payer, &mint_authority],
        svm.latest_blockhash(),
    );
    svm.send_transaction(VersionedTransaction::from(mint_to_tx))
        .unwrap();

    let payer_raw = svm.get_account(&payer.pubkey()).unwrap();
    let mint_raw = svm.get_account(&mint_keypair.pubkey()).unwrap();
    let token_raw = svm.get_account(&token_keypair.pubkey()).unwrap();

    let accounts = vec![
        AccountData {
            meta: AccountMeta { key: payer.pubkey(), is_signer: true, is_writable: true },
            lamports: payer_raw.lamports,
            data: payer_raw.data.to_vec(),
            owner: payer_raw.owner,
            executable: payer_raw.executable,
            rent_epoch: payer_raw.rent_epoch,
        },
        AccountData {
            meta: AccountMeta { key: mint_keypair.pubkey(), is_signer: false, is_writable: true },
            lamports: mint_raw.lamports,
            data: mint_raw.data.to_vec(),
            owner: mint_raw.owner,
            executable: mint_raw.executable,
            rent_epoch: mint_raw.rent_epoch,
        },
        AccountData {
            meta: AccountMeta { key: token_keypair.pubkey(), is_signer: false, is_writable: true },
            lamports: token_raw.lamports,
            data: token_raw.data.to_vec(),
            owner: token_raw.owner,
            executable: token_raw.executable,
            rent_epoch: token_raw.rent_epoch,
        },
    ];

    let mut bytes = Vec::new();
    for account in &accounts {
        account.serialize(&mut bytes);
    }

    let builder = InstructionAccountsBuilder::from_bytes(&bytes);
    InstructionAccounts::init_from_builder(builder);

    for expected in &accounts {
        let info = cvlr_new_account_info();

        assert_eq!(*info.key, expected.meta.key);
        assert_eq!(*info.owner, expected.owner);
        assert_eq!(info.lamports(), expected.lamports);
        assert_eq!(&**info.data.borrow(), expected.data.as_slice());
        assert_eq!(info.is_signer, expected.meta.is_signer);
        assert_eq!(info.is_writable, expected.meta.is_writable);
        assert_eq!(info.executable, expected.executable);
        assert_eq!(info.rent_epoch, expected.rent_epoch);
    }
}
