use {
    crate::common::{
        TWAP_INITIAL_OBSERVATION, TWAP_MAX_OBSERVATION_CHANGE_PER_UPDATE, TWAP_START_DELAY_SLOTS,
    },
    litesvm::LiteSVM,
    lobbyist::amm_cpi::{AddLiquidityArgs, CreateAmmArgs},
    solana_instruction::{AccountMeta, Instruction},
    solana_keypair::Keypair,
    solana_program::{pubkey::Pubkey, system_program},
    solana_signer::Signer,
    solana_transaction::Transaction,
    spl_associated_token_account::{
        get_associated_token_address, instruction::create_associated_token_account_idempotent,
    },
};

pub const AMM_PROGRAM_ID: Pubkey =
    Pubkey::from_str_const("AMMJdEiCCa8mdugg6JPF7gFirmmxisTfDJoSNSUi5zDJ");

const CREATE_AMM_DISCRIMINATOR: &[u8] = &[242, 91, 21, 170, 5, 68, 125, 64];
const ADD_LIQUIDITY_DISCRIMINATOR: &[u8] = &[181, 157, 89, 67, 143, 182, 52, 72];

pub fn amm_pda(base_mint: Pubkey, quote_mint: Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[b"amm__", base_mint.as_ref(), quote_mint.as_ref()],
        &AMM_PROGRAM_ID,
    )
    .0
}

pub fn amm_lp_mint(amm_pda: Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"amm_lp_mint", amm_pda.as_ref()], &AMM_PROGRAM_ID).0
}

pub fn amm_event_authority_pda() -> Pubkey {
    Pubkey::find_program_address(&[b"__event_authority"], &AMM_PROGRAM_ID).0
}

pub fn create_amm(
    svm: &mut LiteSVM,
    signer: &Keypair,
    base_mint: Pubkey,
    quote_mint: Pubkey,
) -> (Pubkey, Pubkey) {
    let amm_event_authority_pda = amm_event_authority_pda();
    let amm_pda = amm_pda(base_mint, quote_mint);
    let amm_lp_mint = amm_lp_mint(amm_pda);
    let base_vault_ata = get_associated_token_address(&amm_pda, &base_mint);
    let quote_vault_ata = get_associated_token_address(&amm_pda, &quote_mint);
    let mut create_amm_ix_data = CREATE_AMM_DISCRIMINATOR.to_vec();
    borsh::to_writer(
        &mut create_amm_ix_data,
        &CreateAmmArgs {
            twap_initial_observation: TWAP_INITIAL_OBSERVATION as u128,
            twap_max_observation_change_per_update: TWAP_MAX_OBSERVATION_CHANGE_PER_UPDATE as u128,
            twap_start_delay_slots: TWAP_START_DELAY_SLOTS,
        },
    )
    .unwrap();
    let create_amm_ix = Instruction::new_with_bytes(
        AMM_PROGRAM_ID,
        &create_amm_ix_data,
        vec![
            AccountMeta::new(signer.pubkey(), true),
            AccountMeta::new(amm_pda, false),
            AccountMeta::new(amm_lp_mint, false),
            AccountMeta::new_readonly(base_mint, false),
            AccountMeta::new_readonly(quote_mint, false),
            AccountMeta::new(base_vault_ata, false),
            AccountMeta::new(quote_vault_ata, false),
            AccountMeta::new_readonly(spl_associated_token_account::ID, false),
            AccountMeta::new_readonly(spl_token::ID, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(amm_event_authority_pda, false),
            AccountMeta::new_readonly(AMM_PROGRAM_ID, false),
        ],
    );

    let tx = Transaction::new_signed_with_payer(
        &[create_amm_ix],
        Some(&signer.pubkey()),
        &[&signer],
        svm.latest_blockhash(),
    );
    svm.send_transaction(tx).unwrap();

    (amm_pda, amm_lp_mint)
}

pub fn add_liquidity(
    svm: &mut LiteSVM,
    signer: &Keypair,
    amm_pda: Pubkey,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    args: AddLiquidityArgs,
) {
    let amm_event_authority_pda = amm_event_authority_pda();
    let lp_mint = amm_lp_mint(amm_pda);
    let lp_ata = get_associated_token_address(&signer.pubkey(), &lp_mint);
    let base_ata = get_associated_token_address(&signer.pubkey(), &base_mint);
    let quote_ata = get_associated_token_address(&signer.pubkey(), &quote_mint);
    let vault_base_ata = get_associated_token_address(&amm_pda, &base_mint);
    let vault_quote_ata = get_associated_token_address(&amm_pda, &quote_mint);
    let mut add_liquidity_ix_data = ADD_LIQUIDITY_DISCRIMINATOR.to_vec();
    borsh::to_writer(&mut add_liquidity_ix_data, &args).unwrap();
    let add_liquidity_ix = Instruction::new_with_bytes(
        AMM_PROGRAM_ID,
        &add_liquidity_ix_data,
        vec![
            AccountMeta::new(signer.pubkey(), true),
            AccountMeta::new(amm_pda, false),
            AccountMeta::new(lp_mint, false),
            AccountMeta::new(lp_ata, false),
            AccountMeta::new(base_ata, false),
            AccountMeta::new(quote_ata, false),
            AccountMeta::new(vault_base_ata, false),
            AccountMeta::new(vault_quote_ata, false),
            AccountMeta::new_readonly(spl_token::ID, false),
            AccountMeta::new_readonly(amm_event_authority_pda, false),
            AccountMeta::new_readonly(AMM_PROGRAM_ID, false),
        ],
    );

    let tx = Transaction::new_signed_with_payer(
        &[
            create_associated_token_account_idempotent(
                &signer.pubkey(),
                &signer.pubkey(),
                &lp_mint,
                &spl_token::ID,
            ),
            create_associated_token_account_idempotent(
                &signer.pubkey(),
                &amm_pda,
                &base_mint,
                &spl_token::ID,
            ),
            create_associated_token_account_idempotent(
                &signer.pubkey(),
                &amm_pda,
                &quote_mint,
                &spl_token::ID,
            ),
            add_liquidity_ix,
        ],
        Some(&signer.pubkey()),
        &[&signer],
        svm.latest_blockhash(),
    );
    svm.send_transaction(tx).unwrap();
}
