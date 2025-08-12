use {
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_program::{program_pack::Pack, rent::Rent, system_instruction},
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_transaction::Transaction,
    spl_associated_token_account::instruction::create_associated_token_account,
    spl_token::{
        instruction::{initialize_mint2, mint_to_checked},
        state::Mint,
    },
};

pub fn create_token(svm: &mut LiteSVM, signer: &Keypair, initial_supply: u64) -> Pubkey {
    let token_kp = Keypair::new();
    let allocate_token_ix = system_instruction::create_account(
        &signer.pubkey(),
        &token_kp.pubkey(),
        svm.get_sysvar::<Rent>().minimum_balance(Mint::LEN),
        Mint::LEN as u64,
        &spl_token::ID,
    );
    let create_token_ix = initialize_mint2(
        &spl_token::ID,
        &token_kp.pubkey(),
        &signer.pubkey(),
        None,
        6,
    )
    .unwrap();
    let signer_token_ata = spl_associated_token_account::get_associated_token_address(
        &signer.pubkey(),
        &token_kp.pubkey(),
    );
    let initialize_token_account_ix = create_associated_token_account(
        &signer.pubkey(),
        &signer.pubkey(),
        &token_kp.pubkey(),
        &spl_token::ID,
    );
    let mint_token_ix = mint_to_checked(
        &spl_token::ID,
        &token_kp.pubkey(),
        &signer_token_ata,
        &signer.pubkey(),
        &[&signer.pubkey()],
        initial_supply,
        6,
    )
    .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[
            allocate_token_ix,
            create_token_ix,
            initialize_token_account_ix,
            mint_token_ix,
        ],
        Some(&signer.pubkey()),
        &[&signer, &token_kp],
        svm.latest_blockhash(),
    );
    svm.send_transaction(tx).unwrap();

    token_kp.pubkey()
}
