use {
    crate::common::proposal_pda,
    borsh::{BorshDeserialize, BorshSerialize},
    litesvm::LiteSVM,
    solana_instruction::{AccountMeta, Instruction},
    solana_keypair::Keypair,
    solana_program::{pubkey::Pubkey, system_program},
    solana_signer::Signer,
    solana_transaction::Transaction,
    spl_associated_token_account::{
        get_associated_token_address, instruction::create_associated_token_account,
    },
};

pub const CONDITIONAL_VAULT_PROGRAM_ID: Pubkey =
    Pubkey::from_str_const("VLTX1ishMBbcX3rdBWGssxawAo1Q2X2qxYFYqiGodVg");

const CREATE_VAULT_DISCRIMINATOR: &[u8] = &[37, 88, 250, 212, 54, 218, 227, 175];
const SPLIT_TOKENS_DISCRIMINATOR: &[u8] = &[79, 195, 116, 0, 140, 176, 73, 179];

#[derive(BorshSerialize, BorshDeserialize)]
pub struct InitializeQuestionArgs {
    pub question_id: [u8; 32],
    pub oracle: Pubkey,
    pub num_outcomes: u8,
}

pub fn question_pda(question_id: Pubkey, oracle: Pubkey, num_outcomes: u8) -> Pubkey {
    Pubkey::find_program_address(
        &[
            b"question",
            question_id.as_ref(),
            oracle.as_ref(),
            &num_outcomes.to_le_bytes(),
        ],
        &CONDITIONAL_VAULT_PROGRAM_ID,
    )
    .0
}

pub fn vault_pda(question: Pubkey, underlying_mint: Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[
            b"conditional_vault",
            question.as_ref(),
            underlying_mint.as_ref(),
        ],
        &CONDITIONAL_VAULT_PROGRAM_ID,
    )
    .0
}

pub fn conditional_mint_pda(vault: Pubkey, index: u8) -> Pubkey {
    Pubkey::find_program_address(
        &[b"conditional_token", vault.as_ref(), &index.to_le_bytes()],
        &CONDITIONAL_VAULT_PROGRAM_ID,
    )
    .0
}

pub fn conditional_vault_event_authority_pda() -> Pubkey {
    Pubkey::find_program_address(&[b"__event_authority"], &CONDITIONAL_VAULT_PROGRAM_ID).0
}

pub fn create_question(svm: &mut LiteSVM, signer: &Keypair, squads_proposal_pda: Pubkey) -> Pubkey {
    let proposal_pda = proposal_pda(squads_proposal_pda);
    let question_pda = question_pda(proposal_pda, proposal_pda, 2);
    let conditional_vault_event_authority = conditional_vault_event_authority_pda();
    let mut create_question_ix_data = vec![245, 151, 106, 188, 88, 44, 65, 212];
    borsh::to_writer(
        &mut create_question_ix_data,
        &InitializeQuestionArgs {
            question_id: proposal_pda.to_bytes(),
            oracle: proposal_pda,
            num_outcomes: 2,
        },
    )
    .unwrap();
    let create_question_ix = Instruction::new_with_bytes(
        CONDITIONAL_VAULT_PROGRAM_ID,
        &create_question_ix_data,
        vec![
            AccountMeta::new(question_pda, false),
            AccountMeta::new(signer.pubkey(), true),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(conditional_vault_event_authority, false),
            AccountMeta::new_readonly(CONDITIONAL_VAULT_PROGRAM_ID, false),
        ],
    );

    let tx = Transaction::new_signed_with_payer(
        &[create_question_ix],
        Some(&signer.pubkey()),
        &[&signer],
        svm.latest_blockhash(),
    );
    svm.send_transaction(tx).unwrap();

    question_pda
}

pub fn create_conditional_vault(
    svm: &mut LiteSVM,
    signer: &Keypair,
    question_pda: Pubkey,
    token_mint: Pubkey,
) -> (Pubkey, Pubkey, Pubkey) {
    let token_vault_pda = vault_pda(question_pda, token_mint);
    let token_vault_ata = get_associated_token_address(&token_vault_pda, &token_mint);
    let fail_token_mint = conditional_mint_pda(token_vault_pda, 0);
    let pass_token_mint = conditional_mint_pda(token_vault_pda, 1);
    let create_token_vault_ix_data = CREATE_VAULT_DISCRIMINATOR.to_vec();
    let create_token_vault_ix = Instruction::new_with_bytes(
        CONDITIONAL_VAULT_PROGRAM_ID,
        &create_token_vault_ix_data,
        vec![
            AccountMeta::new(token_vault_pda, false),
            AccountMeta::new(question_pda, false),
            AccountMeta::new_readonly(token_mint, false),
            AccountMeta::new_readonly(token_vault_ata, false),
            AccountMeta::new(signer.pubkey(), true),
            AccountMeta::new_readonly(spl_token::ID, false),
            AccountMeta::new_readonly(spl_associated_token_account::ID, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(conditional_vault_event_authority_pda(), false),
            AccountMeta::new_readonly(CONDITIONAL_VAULT_PROGRAM_ID, false),
            AccountMeta::new(fail_token_mint, false),
            AccountMeta::new(pass_token_mint, false),
        ],
    );

    let tx = Transaction::new_signed_with_payer(
        &[
            create_associated_token_account(
                &signer.pubkey(),
                &token_vault_pda,
                &token_mint,
                &spl_token::ID,
            ),
            create_token_vault_ix,
        ],
        Some(&signer.pubkey()),
        &[&signer],
        svm.latest_blockhash(),
    );
    svm.send_transaction(tx).unwrap();

    (token_vault_pda, fail_token_mint, pass_token_mint)
}

pub fn split_tokens(
    svm: &mut LiteSVM,
    signer: &Keypair,
    question_pda: Pubkey,
    token_vault_pda: Pubkey,
    token_mint: Pubkey,
    amount: u64,
    base_mint: Pubkey,
    quote_mint: Pubkey,
) -> (Pubkey, Pubkey) {
    let token_vault_ata = get_associated_token_address(&token_vault_pda, &token_mint);
    let base_ata = get_associated_token_address(&signer.pubkey(), &base_mint);
    let quote_ata = get_associated_token_address(&signer.pubkey(), &quote_mint);
    let mut split_tokens_ix_data = SPLIT_TOKENS_DISCRIMINATOR.to_vec();
    split_tokens_ix_data.extend(amount.to_le_bytes());
    let split_tokens_ix = Instruction::new_with_bytes(
        CONDITIONAL_VAULT_PROGRAM_ID,
        &split_tokens_ix_data,
        vec![
            AccountMeta::new_readonly(question_pda, false),
            AccountMeta::new(token_vault_pda, false),
            AccountMeta::new(token_vault_ata, false),
            AccountMeta::new_readonly(signer.pubkey(), true),
            AccountMeta::new(
                get_associated_token_address(&signer.pubkey(), &token_mint),
                false,
            ),
            AccountMeta::new_readonly(spl_token::ID, false),
            AccountMeta::new_readonly(conditional_vault_event_authority_pda(), false),
            AccountMeta::new_readonly(CONDITIONAL_VAULT_PROGRAM_ID, false),
            AccountMeta::new(base_mint, false),
            AccountMeta::new(quote_mint, false),
            AccountMeta::new(base_ata, false),
            AccountMeta::new(quote_ata, false),
        ],
    );

    let tx = Transaction::new_signed_with_payer(
        &[
            create_associated_token_account(
                &signer.pubkey(),
                &signer.pubkey(),
                &base_mint,
                &spl_token::ID,
            ),
            create_associated_token_account(
                &signer.pubkey(),
                &signer.pubkey(),
                &quote_mint,
                &spl_token::ID,
            ),
            split_tokens_ix,
        ],
        Some(&signer.pubkey()),
        &[&signer],
        svm.latest_blockhash(),
    );
    svm.send_transaction(tx).unwrap();

    (base_ata, quote_ata)
}
