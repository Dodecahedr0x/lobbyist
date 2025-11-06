use {
    crate::common::{
        create_conditional_vault, create_dao, create_proposal, create_question,
        create_squads_proposal, create_token, launch_proposal, provide_liquidity, split_tokens,
        squads::{SQUADS_PROGRAM_CONFIG, SQUADS_PROGRAM_ID},
        CONDITIONAL_VAULT_PROGRAM_ID, FUTARCHY_PROGRAM_ID,
    },
    lazy_static::lazy_static,
    litesvm::LiteSVM,
    lobbyist::futarchy_cpi::ProvideLiquidityParams,
    solana_account::{Account, WritableAccount},
    solana_keypair::Keypair,
    solana_native_token::LAMPORTS_PER_SOL,
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    std::{fs, path::PathBuf},
};

// pub const TWAP_INITIAL_OBSERVATION: u64 = 1_000_000_000_000;
// pub const TWAP_MAX_OBSERVATION_CHANGE_PER_UPDATE: u64 = 20_000_000_000;
// pub const TWAP_START_DELAY_SLOTS: u64 = 0;

lazy_static! {
    pub static ref PERMISSIONLESS_ACCOUNT: Keypair = Keypair::from_bytes(&[
        249, 158, 188, 171, 243, 143, 1, 48, 87, 243, 209, 153, 144, 106, 23, 88, 161, 209, 65,
        217, 199, 121, 0, 250, 3, 203, 133, 138, 141, 112, 243, 38, 198, 205, 120, 222, 160, 224,
        151, 190, 84, 254, 127, 178, 224, 195, 130, 243, 145, 73, 20, 91, 9, 69, 222, 184, 23, 1,
        2, 196, 202, 206, 153, 192,
    ])
    .unwrap();
}

pub struct TestContext {
    pub svm: LiteSVM,
    pub signer: Keypair,
    pub proposal: Pubkey,
    pub question: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub dao: Pubkey,
    pub base_vault_pda: Pubkey,
    pub quote_vault_pda: Pubkey,
    pub pass_base_mint: Pubkey,
    pub pass_quote_mint: Pubkey,
    pub fail_base_mint: Pubkey,
    pub fail_quote_mint: Pubkey,
}

impl TestContext {
    pub fn new(initial_supply: u64) -> TestContext {
        let mut svm = LiteSVM::new();

        // Add programs
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        svm.add_program(
            lobbyist::ID.into(),
            include_bytes!("../../../../target/deploy/lobbyist.so"),
        );
        svm.add_program(
            SQUADS_PROGRAM_ID,
            include_bytes!("../fixtures/squads_multisig.so"),
        );
        svm.add_program(
            FUTARCHY_PROGRAM_ID,
            include_bytes!("../fixtures/futarchy.so"),
        );
        svm.add_program(
            CONDITIONAL_VAULT_PROGRAM_ID,
            include_bytes!("../fixtures/conditional_vault.so"),
        );

        let squads_program_config_account = Account::create(
            LAMPORTS_PER_SOL,
            fs::read(manifest_dir.join("tests/fixtures/squads_program_config")).unwrap(),
            SQUADS_PROGRAM_ID,
            false,
            0,
        );
        svm.set_account(SQUADS_PROGRAM_CONFIG, squads_program_config_account)
            .unwrap();

        let signer = Keypair::from_base58_string("3Lut2ojVRG2ipFZgQptbrcThUsZ4vwAumLGp9fQdUCZKmu5BRr7HLQ5V31M44tqzC7cX17magSozoY8x1SB7AXfS");

        svm.airdrop(&signer.pubkey(), 100 * LAMPORTS_PER_SOL)
            .unwrap();

        let base_mint = create_token(&mut svm, &signer, initial_supply);
        let quote_mint = create_token(&mut svm, &signer, initial_supply);

        let (dao_pda, multisig_pda) = create_dao(&mut svm, &signer, base_mint, quote_mint);

        let squads_proposal_pda = create_squads_proposal(&mut svm, &signer, multisig_pda);

        let question_pda = create_question(&mut svm, &signer, squads_proposal_pda);

        let (base_vault_pda, fail_base_mint, pass_base_mint) =
            create_conditional_vault(&mut svm, &signer, question_pda, base_mint);
        let (quote_vault_pda, fail_quote_mint, pass_quote_mint) =
            create_conditional_vault(&mut svm, &signer, question_pda, quote_mint);

        let (_fail_token_ata, _pass_token_ata) = split_tokens(
            &mut svm,
            &signer,
            question_pda,
            base_vault_pda,
            base_mint,
            initial_supply / 4,
            fail_base_mint,
            pass_base_mint,
        );
        let (_fail_usdc_ata, _pass_usdc_ata) = split_tokens(
            &mut svm,
            &signer,
            question_pda,
            quote_vault_pda,
            quote_mint,
            initial_supply / 4,
            fail_quote_mint,
            pass_quote_mint,
        );

        provide_liquidity(
            &mut svm,
            &signer,
            dao_pda,
            base_mint,
            quote_mint,
            ProvideLiquidityParams {
                quote_amount: initial_supply / 4,
                max_base_amount: initial_supply / 4,
                min_liquidity: 100,
                position_authority: signer.pubkey().to_bytes(),
            },
        );

        let proposal_pda = create_proposal(
            &mut svm,
            &signer,
            dao_pda,
            question_pda,
            squads_proposal_pda,
            base_vault_pda,
            quote_vault_pda,
        );

        launch_proposal(
            &mut svm,
            &signer,
            dao_pda,
            proposal_pda,
            base_vault_pda,
            quote_vault_pda,
            pass_base_mint,
            pass_quote_mint,
            fail_base_mint,
            fail_quote_mint,
        );

        TestContext {
            svm,
            signer,
            dao: dao_pda,
            proposal: proposal_pda,
            question: question_pda,
            base_mint,
            quote_mint,
            base_vault_pda,
            quote_vault_pda,
            pass_base_mint,
            pass_quote_mint,
            fail_base_mint,
            fail_quote_mint,
        }
    }
}
