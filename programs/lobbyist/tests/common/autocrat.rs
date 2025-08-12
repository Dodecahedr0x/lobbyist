use {
    crate::common::{
        squads_multisig_pda, squads_spending_limit_pda, squads_vault_pda, SQUADS_PROGRAM_CONFIG,
        SQUADS_PROGRAM_CONFIG_TREASURY, SQUADS_PROGRAM_ID,
    },
    litesvm::LiteSVM,
    lobbyist::autocrat_cpi::{InitializeDaoParams, InitializeProposalParams},
    solana_instruction::{AccountMeta, Instruction},
    solana_keypair::Keypair,
    solana_program::system_program,
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_transaction::Transaction,
    spl_associated_token_account::get_associated_token_address,
};

pub const AUTOCRAT_PROGRAM_ID: Pubkey =
    Pubkey::from_str_const("auToUr3CQza3D4qreT6Std2MTomfzvrEeCC5qh7ivW5");

pub const CREATE_DAO_DISCRIMINATOR: &[u8] = &[128_u8, 226, 96, 90, 39, 56, 24, 196];
pub const CREATE_PROPOSAL_DISCRIMINATOR: &[u8] = &[50_u8, 73, 156, 98, 129, 149, 21, 158];

pub const MIN_LP_TOKENS_LOCKED: u64 = 100;

pub fn dao_pda(dao_creator: Pubkey, nonce: u64) -> Pubkey {
    Pubkey::find_program_address(
        &[b"dao", dao_creator.as_ref(), nonce.to_le_bytes().as_ref()],
        &AUTOCRAT_PROGRAM_ID,
    )
    .0
}

pub fn autocrat_event_authority_pda() -> Pubkey {
    Pubkey::find_program_address(&[b"__event_authority"], &AUTOCRAT_PROGRAM_ID).0
}

pub fn proposal_pda(squads_proposal: Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[b"proposal", squads_proposal.as_ref()],
        &AUTOCRAT_PROGRAM_ID,
    )
    .0
}

pub fn create_dao(
    svm: &mut LiteSVM,
    signer: &Keypair,
    token_mint: Pubkey,
    usdc_mint: Pubkey,
) -> (Pubkey, Pubkey) {
    let twap_initial_observation = 1_000_000_000_000;
    let twap_max_observation_change_per_update = 20_000_000_000;
    let twap_start_delay_slots = 0;
    let nonce = 1;
    let dao_pda = dao_pda(signer.pubkey(), nonce);
    let multisig_pda = squads_multisig_pda(dao_pda);
    let squads_vault_pda = squads_vault_pda(multisig_pda, 0);
    let spending_limit_pda = squads_spending_limit_pda(&multisig_pda, &dao_pda);
    let autocrat_event_authority_pda = autocrat_event_authority_pda();
    let mut create_dao_ix_data = CREATE_DAO_DISCRIMINATOR.to_vec();
    borsh::to_writer(
        &mut create_dao_ix_data,
        &InitializeDaoParams {
            twap_initial_observation,
            twap_max_observation_change_per_update,
            min_quote_futarchic_liquidity: MIN_LP_TOKENS_LOCKED,
            min_base_futarchic_liquidity: MIN_LP_TOKENS_LOCKED,
            pass_threshold_bps: 100,
            slots_per_proposal: 100,
            initial_spending_limit: None,
            nonce,
            twap_start_delay_slots,
        },
    )
    .unwrap();
    let create_dao_ix = Instruction::new_with_bytes(
        AUTOCRAT_PROGRAM_ID,
        &create_dao_ix_data,
        vec![
            AccountMeta::new(dao_pda, false),
            AccountMeta::new_readonly(signer.pubkey(), true),
            AccountMeta::new(signer.pubkey(), true),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(token_mint, false),
            AccountMeta::new_readonly(usdc_mint, false),
            AccountMeta::new(multisig_pda, false),
            AccountMeta::new_readonly(squads_vault_pda, false),
            AccountMeta::new_readonly(SQUADS_PROGRAM_ID, false),
            AccountMeta::new_readonly(SQUADS_PROGRAM_CONFIG, false),
            AccountMeta::new(SQUADS_PROGRAM_CONFIG_TREASURY, false),
            AccountMeta::new(spending_limit_pda, false),
            AccountMeta::new_readonly(autocrat_event_authority_pda, false),
            AccountMeta::new_readonly(AUTOCRAT_PROGRAM_ID, false),
        ],
    );

    let tx = Transaction::new_signed_with_payer(
        &[create_dao_ix],
        Some(&signer.pubkey()),
        &[&signer],
        svm.latest_blockhash(),
    );
    svm.send_transaction(tx).unwrap();

    (dao_pda, multisig_pda)
}

pub fn create_proposal(
    svm: &mut LiteSVM,
    signer: &Keypair,
    dao_pda: Pubkey,
    question: Pubkey,
    squads_proposal_pda: Pubkey,
    base_vault: Pubkey,
    quote_vault: Pubkey,
    fail_amm_pda: Pubkey,
    pass_amm_pda: Pubkey,
    fail_lp_mint: Pubkey,
    pass_lp_mint: Pubkey,
) -> Pubkey {
    let event_authority_pda = autocrat_event_authority_pda();
    let proposal = proposal_pda(squads_proposal_pda);
    let pass_lp_ata = get_associated_token_address(&signer.pubkey(), &pass_lp_mint);
    let fail_lp_ata = get_associated_token_address(&signer.pubkey(), &fail_lp_mint);
    let pass_lp_vault_ata = get_associated_token_address(&proposal, &pass_lp_mint);
    let fail_lp_vault_ata = get_associated_token_address(&proposal, &fail_lp_mint);
    let mut create_proposal_ix_data = CREATE_PROPOSAL_DISCRIMINATOR.to_vec();
    borsh::to_writer(
        &mut create_proposal_ix_data,
        &InitializeProposalParams {
            description_url: "".to_string(),
            pass_lp_tokens_to_lock: MIN_LP_TOKENS_LOCKED,
            fail_lp_tokens_to_lock: MIN_LP_TOKENS_LOCKED,
        },
    )
    .unwrap();
    let create_proposal_ix = Instruction::new_with_bytes(
        AUTOCRAT_PROGRAM_ID,
        &create_proposal_ix_data,
        vec![
            AccountMeta::new(proposal, false),
            AccountMeta::new_readonly(squads_proposal_pda, false),
            AccountMeta::new(dao_pda, false),
            AccountMeta::new_readonly(question, false),
            AccountMeta::new_readonly(quote_vault, false),
            AccountMeta::new_readonly(base_vault, false),
            AccountMeta::new_readonly(pass_amm_pda, false),
            AccountMeta::new_readonly(pass_lp_mint, false),
            AccountMeta::new_readonly(fail_lp_mint, false),
            AccountMeta::new_readonly(fail_amm_pda, false),
            AccountMeta::new(pass_lp_ata, false),
            AccountMeta::new(fail_lp_ata, false),
            AccountMeta::new(pass_lp_vault_ata, false),
            AccountMeta::new(fail_lp_vault_ata, false),
            AccountMeta::new_readonly(signer.pubkey(), true),
            AccountMeta::new(signer.pubkey(), true),
            AccountMeta::new_readonly(spl_token::ID, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(spl_associated_token_account::ID, false),
            AccountMeta::new_readonly(event_authority_pda, false),
            AccountMeta::new_readonly(AUTOCRAT_PROGRAM_ID, false),
        ],
    );

    let tx = Transaction::new_signed_with_payer(
        &[create_proposal_ix],
        Some(&signer.pubkey()),
        &[&signer],
        svm.latest_blockhash(),
    );
    svm.send_transaction(tx).unwrap();

    proposal
}
