use {
    crate::{
        assert_tx,
        common::{
            conditional_vault_event_authority_pda, squads_multisig_pda, squads_spending_limit_pda,
            squads_vault_pda, CONDITIONAL_VAULT_PROGRAM_ID, SQUADS_PROGRAM_CONFIG,
            SQUADS_PROGRAM_CONFIG_TREASURY, SQUADS_PROGRAM_ID,
        },
    },
    litesvm::LiteSVM,
    lobbyist::futarchy_cpi::{
        ConditionalSwapParams, InitializeDaoParams, Market, ProvideLiquidityParams, SwapType,
    },
    solana_instruction::{AccountMeta, Instruction},
    solana_keypair::Keypair,
    solana_program::system_program,
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_transaction::Transaction,
    spl_associated_token_account::get_associated_token_address,
};

pub const FUTARCHY_PROGRAM_ID: Pubkey =
    Pubkey::from_str_const("FUTARELBfJfQ8RDGhg1wdhddq1odMAJUePHFuBYfUxKq");

pub const CREATE_DAO_DISCRIMINATOR: &[u8] = &[128, 226, 96, 90, 39, 56, 24, 196];
pub const CREATE_PROPOSAL_DISCRIMINATOR: &[u8] = &[50, 73, 156, 98, 129, 149, 21, 158];
pub const LAUNCH_PROPOSAL_DISCRIMINATOR: &[u8] = &[16, 211, 189, 119, 245, 72, 0, 229];
pub const PROVIDE_LIQUIDITY_DISCRIMINATOR: &[u8] = &[40, 110, 107, 116, 174, 127, 97, 204];
pub const CONDITIONAL_SWAP_DISCRIMINATOR: &[u8] = &[194, 136, 220, 89, 242, 169, 130, 157];

pub const MIN_LP_TOKENS_LOCKED: u64 = 100;

pub fn dao_pda(dao_creator: Pubkey, nonce: u64) -> Pubkey {
    Pubkey::find_program_address(
        &[b"dao", dao_creator.as_ref(), nonce.to_le_bytes().as_ref()],
        &FUTARCHY_PROGRAM_ID,
    )
    .0
}

pub fn futarchy_event_authority_pda() -> Pubkey {
    Pubkey::find_program_address(&[b"__event_authority"], &FUTARCHY_PROGRAM_ID).0
}

pub fn proposal_pda(squads_proposal: Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[b"proposal", squads_proposal.as_ref()],
        &FUTARCHY_PROGRAM_ID,
    )
    .0
}

pub fn amm_position_pda(dao_pda: Pubkey, position_authority: Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[
            b"amm_position",
            dao_pda.as_ref(),
            position_authority.as_ref(),
        ],
        &FUTARCHY_PROGRAM_ID,
    )
    .0
}

pub fn create_dao(
    svm: &mut LiteSVM,
    signer: &Keypair,
    base_mint: Pubkey,
    quote_mint: Pubkey,
) -> (Pubkey, Pubkey) {
    let twap_initial_observation = 1_000_000_000_000;
    let twap_max_observation_change_per_update = 20_000_000_000;
    let twap_start_delay_seconds = 0;
    let nonce = 1;
    let dao_pda = dao_pda(signer.pubkey(), nonce);
    let multisig_pda = squads_multisig_pda(dao_pda);
    let squads_vault_pda = squads_vault_pda(multisig_pda, 0);
    let spending_limit_pda = squads_spending_limit_pda(&multisig_pda, &dao_pda);
    let futarchy_amm_base_vault = get_associated_token_address(&dao_pda, &base_mint);
    let futarchy_amm_quote_vault = get_associated_token_address(&dao_pda, &quote_mint);
    let futarchy_event_authority_pda = futarchy_event_authority_pda();
    let mut create_dao_ix_data = CREATE_DAO_DISCRIMINATOR.to_vec();
    borsh::to_writer(
        &mut create_dao_ix_data,
        &InitializeDaoParams {
            twap_initial_observation,
            twap_max_observation_change_per_update,
            min_quote_futarchic_liquidity: MIN_LP_TOKENS_LOCKED,
            min_base_futarchic_liquidity: MIN_LP_TOKENS_LOCKED,
            pass_threshold_bps: 100,
            seconds_per_proposal: 86400,
            initial_spending_limit: None,
            nonce,
            twap_start_delay_seconds,
            base_to_stake: 0,
        },
    )
    .unwrap();
    let create_dao_ix = Instruction::new_with_bytes(
        FUTARCHY_PROGRAM_ID,
        &create_dao_ix_data,
        vec![
            AccountMeta::new(dao_pda, false),
            AccountMeta::new_readonly(signer.pubkey(), true),
            AccountMeta::new(signer.pubkey(), true),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(base_mint, false),
            AccountMeta::new_readonly(quote_mint, false),
            AccountMeta::new(multisig_pda, false),
            AccountMeta::new_readonly(squads_vault_pda, false),
            AccountMeta::new_readonly(SQUADS_PROGRAM_ID, false),
            AccountMeta::new_readonly(SQUADS_PROGRAM_CONFIG, false),
            AccountMeta::new(SQUADS_PROGRAM_CONFIG_TREASURY, false),
            AccountMeta::new(spending_limit_pda, false),
            AccountMeta::new(futarchy_amm_base_vault, false),
            AccountMeta::new(futarchy_amm_quote_vault, false),
            AccountMeta::new_readonly(spl_token::ID, false),
            AccountMeta::new_readonly(spl_associated_token_account::ID, false),
            AccountMeta::new_readonly(futarchy_event_authority_pda, false),
            AccountMeta::new_readonly(FUTARCHY_PROGRAM_ID, false),
        ],
    );

    let tx = Transaction::new_signed_with_payer(
        &[create_dao_ix],
        Some(&signer.pubkey()),
        &[&signer],
        svm.latest_blockhash(),
    );
    assert_tx!(svm.send_transaction(tx));

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
) -> Pubkey {
    let event_authority_pda = futarchy_event_authority_pda();
    let proposal = proposal_pda(squads_proposal_pda);
    let create_proposal_ix_data = CREATE_PROPOSAL_DISCRIMINATOR.to_vec();
    let create_proposal_ix = Instruction::new_with_bytes(
        FUTARCHY_PROGRAM_ID,
        &create_proposal_ix_data,
        vec![
            AccountMeta::new(proposal, false),
            AccountMeta::new_readonly(squads_proposal_pda, false),
            AccountMeta::new(dao_pda, false),
            AccountMeta::new_readonly(question, false),
            AccountMeta::new_readonly(quote_vault, false),
            AccountMeta::new_readonly(base_vault, false),
            AccountMeta::new_readonly(signer.pubkey(), true),
            AccountMeta::new(signer.pubkey(), true),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(event_authority_pda, false),
            AccountMeta::new_readonly(FUTARCHY_PROGRAM_ID, false),
        ],
    );

    let tx = Transaction::new_signed_with_payer(
        &[create_proposal_ix],
        Some(&signer.pubkey()),
        &[&signer],
        svm.latest_blockhash(),
    );
    assert_tx!(svm.send_transaction(tx));

    proposal
}

pub fn launch_proposal(
    svm: &mut LiteSVM,
    signer: &Keypair,
    dao_pda: Pubkey,
    proposal: Pubkey,
    base_vault: Pubkey,
    quote_vault: Pubkey,
    pass_base_mint: Pubkey,
    pass_quote_mint: Pubkey,
    fail_base_mint: Pubkey,
    fail_quote_mint: Pubkey,
) -> Pubkey {
    let amm_pass_base_vault = get_associated_token_address(&dao_pda, &pass_base_mint);
    let amm_pass_quote_vault = get_associated_token_address(&dao_pda, &pass_quote_mint);
    let amm_fail_base_vault = get_associated_token_address(&dao_pda, &fail_base_mint);
    let amm_fail_quote_vault = get_associated_token_address(&dao_pda, &fail_quote_mint);
    let event_authority_pda = futarchy_event_authority_pda();
    let create_proposal_ix_data = LAUNCH_PROPOSAL_DISCRIMINATOR.to_vec();
    let create_proposal_ix = Instruction::new_with_bytes(
        FUTARCHY_PROGRAM_ID,
        &create_proposal_ix_data,
        vec![
            AccountMeta::new(proposal, false),
            AccountMeta::new_readonly(base_vault, false),
            AccountMeta::new_readonly(quote_vault, false),
            AccountMeta::new_readonly(pass_base_mint, false),
            AccountMeta::new_readonly(pass_quote_mint, false),
            AccountMeta::new_readonly(fail_base_mint, false),
            AccountMeta::new_readonly(fail_quote_mint, false),
            AccountMeta::new(dao_pda, false),
            AccountMeta::new(signer.pubkey(), true),
            AccountMeta::new(amm_pass_base_vault, false),
            AccountMeta::new(amm_pass_quote_vault, false),
            AccountMeta::new(amm_fail_base_vault, false),
            AccountMeta::new(amm_fail_quote_vault, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(spl_token::ID, false),
            AccountMeta::new_readonly(spl_associated_token_account::ID, false),
            AccountMeta::new_readonly(event_authority_pda, false),
            AccountMeta::new_readonly(FUTARCHY_PROGRAM_ID, false),
        ],
    );

    let tx = Transaction::new_signed_with_payer(
        &[create_proposal_ix],
        Some(&signer.pubkey()),
        &[&signer],
        svm.latest_blockhash(),
    );
    assert_tx!(svm.send_transaction(tx));

    proposal
}

pub fn provide_liquidity(
    svm: &mut LiteSVM,
    signer: &Keypair,
    dao_pda: Pubkey,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    params: ProvideLiquidityParams,
) {
    let mut provide_liquidity_ix_data = PROVIDE_LIQUIDITY_DISCRIMINATOR.to_vec();
    borsh::to_writer(&mut provide_liquidity_ix_data, &params).unwrap();
    let liquidity_provider_base_account =
        get_associated_token_address(&signer.pubkey(), &base_mint);
    let liquidity_provider_quote_account =
        get_associated_token_address(&signer.pubkey(), &quote_mint);
    let amm_base_vault = get_associated_token_address(&dao_pda, &base_mint);
    let amm_quote_vault = get_associated_token_address(&dao_pda, &quote_mint);
    let amm_position_pda = amm_position_pda(dao_pda, signer.pubkey());
    let event_authority_pda = futarchy_event_authority_pda();
    let provide_liquidity_ix = Instruction::new_with_bytes(
        FUTARCHY_PROGRAM_ID,
        &provide_liquidity_ix_data,
        vec![
            AccountMeta::new(dao_pda, false),
            AccountMeta::new_readonly(signer.pubkey(), true),
            AccountMeta::new(liquidity_provider_base_account, false),
            AccountMeta::new(liquidity_provider_quote_account, false),
            AccountMeta::new(signer.pubkey(), true),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new(amm_base_vault, false),
            AccountMeta::new(amm_quote_vault, false),
            AccountMeta::new(amm_position_pda, false),
            AccountMeta::new_readonly(spl_token::ID, false),
            AccountMeta::new_readonly(event_authority_pda, false),
            AccountMeta::new_readonly(FUTARCHY_PROGRAM_ID, false),
        ],
    );

    let tx = Transaction::new_signed_with_payer(
        &[provide_liquidity_ix],
        Some(&signer.pubkey()),
        &[&signer],
        svm.latest_blockhash(),
    );
    assert_tx!(svm.send_transaction(tx));
}

pub fn conditional_swap(
    svm: &mut LiteSVM,
    trader: &Keypair,
    dao_pda: Pubkey,
    proposal: Pubkey,
    question: Pubkey,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    base_vault: Pubkey,
    quote_vault: Pubkey,
    pass_base_mint: Pubkey,
    pass_quote_mint: Pubkey,
    fail_base_mint: Pubkey,
    fail_quote_mint: Pubkey,
    params: ConditionalSwapParams,
) {
    let amm_base_vault = get_associated_token_address(&dao_pda, &base_mint);
    let amm_quote_vault = get_associated_token_address(&dao_pda, &quote_mint);
    let amm_pass_base_vault = get_associated_token_address(&dao_pda, &pass_base_mint);
    let amm_pass_quote_vault = get_associated_token_address(&dao_pda, &pass_quote_mint);
    let amm_fail_base_vault = get_associated_token_address(&dao_pda, &fail_base_mint);
    let amm_fail_quote_vault = get_associated_token_address(&dao_pda, &fail_quote_mint);

    let (trader_input_account, trader_output_account) = match (&params.swap_type, &params.market) {
        (SwapType::Buy, Market::Pass) => (
            get_associated_token_address(&trader.pubkey(), &pass_quote_mint),
            get_associated_token_address(&trader.pubkey(), &pass_base_mint),
        ),
        (SwapType::Buy, Market::Fail) => (
            get_associated_token_address(&trader.pubkey(), &pass_base_mint),
            get_associated_token_address(&trader.pubkey(), &pass_quote_mint),
        ),
        (SwapType::Sell, Market::Pass) => (
            get_associated_token_address(&trader.pubkey(), &fail_base_mint),
            get_associated_token_address(&trader.pubkey(), &fail_quote_mint),
        ),
        (SwapType::Sell, Market::Fail) => (
            get_associated_token_address(&trader.pubkey(), &fail_quote_mint),
            get_associated_token_address(&trader.pubkey(), &fail_base_mint),
        ),
        (_, Market::Spot) => unreachable!(),
    };

    let base_vault_underlying_token_account = get_associated_token_address(&base_vault, &base_mint);
    let quote_vault_underlying_token_account =
        get_associated_token_address(&quote_vault, &quote_mint);
    let vault_event_authority = conditional_vault_event_authority_pda();
    let event_authority_pda = futarchy_event_authority_pda();

    let mut conditional_swap_ix_data = CONDITIONAL_SWAP_DISCRIMINATOR.to_vec();
    borsh::to_writer(&mut conditional_swap_ix_data, &params).unwrap();
    let conditional_swap_ix = Instruction::new_with_bytes(
        FUTARCHY_PROGRAM_ID,
        &conditional_swap_ix_data,
        vec![
            AccountMeta::new(dao_pda, false),
            AccountMeta::new(amm_base_vault, false),
            AccountMeta::new(amm_quote_vault, false),
            AccountMeta::new_readonly(proposal, false),
            AccountMeta::new(amm_pass_base_vault, false),
            AccountMeta::new(amm_pass_quote_vault, false),
            AccountMeta::new(amm_fail_base_vault, false),
            AccountMeta::new(amm_fail_quote_vault, false),
            AccountMeta::new_readonly(trader.pubkey(), true),
            AccountMeta::new(trader_input_account, false),
            AccountMeta::new(trader_output_account, false),
            AccountMeta::new(base_vault, false),
            AccountMeta::new(base_vault_underlying_token_account, false),
            AccountMeta::new(quote_vault, false),
            AccountMeta::new(quote_vault_underlying_token_account, false),
            AccountMeta::new(pass_base_mint, false),
            AccountMeta::new(fail_base_mint, false),
            AccountMeta::new(pass_quote_mint, false),
            AccountMeta::new(fail_quote_mint, false),
            AccountMeta::new_readonly(CONDITIONAL_VAULT_PROGRAM_ID, false),
            AccountMeta::new_readonly(vault_event_authority, false),
            AccountMeta::new_readonly(question, false),
            AccountMeta::new_readonly(spl_token::ID, false),
            AccountMeta::new_readonly(event_authority_pda, false),
            AccountMeta::new_readonly(FUTARCHY_PROGRAM_ID, false),
        ],
    );

    let tx = Transaction::new_signed_with_payer(
        &[conditional_swap_ix],
        Some(&trader.pubkey()),
        &[&trader],
        svm.latest_blockhash(),
    );
    assert_tx!(svm.send_transaction(tx));
}
