use {
    crate::common::{
        add_liquidity,
        autocrat::AUTOCRAT_PROGRAM_ID,
        create_amm, create_conditional_vault, create_dao, create_proposal, create_question,
        create_squads_proposal, create_token, split_tokens,
        squads::{SQUADS_PROGRAM_CONFIG, SQUADS_PROGRAM_ID},
        AMM_PROGRAM_ID, CONDITIONAL_VAULT_PROGRAM_ID,
    },
    base64::{prelude::BASE64_STANDARD, Engine},
    lazy_static::lazy_static,
    litesvm::LiteSVM,
    lobbyist::amm_cpi::AddLiquidityArgs,
    pyth_min::byte_utils::hex_to_bytes,
    solana_account::{Account, WritableAccount},
    solana_keypair::Keypair,
    solana_native_token::LAMPORTS_PER_SOL,
    solana_pubkey::{pubkey, Pubkey},
    solana_signer::Signer,
    std::{fs, path::PathBuf},
};

pub const TWAP_INITIAL_OBSERVATION: u64 = 1_000_000_000_000;
pub const TWAP_MAX_OBSERVATION_CHANGE_PER_UPDATE: u64 = 20_000_000_000;
pub const TWAP_START_DELAY_SLOTS: u64 = 0;

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
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub fail_amm: Pubkey,
    pub pass_amm: Pubkey,
    pub fail_base_mint: Pubkey,
    pub fail_quote_mint: Pubkey,
    pub pass_base_mint: Pubkey,
    pub pass_quote_mint: Pubkey,
    pub dao: Pubkey,
    pub price_account: Pubkey,
    pub price_feed_id: [u8; 32],
}

impl TestContext {
    pub fn new(initial_supply: u64) -> TestContext {
        let mut svm = LiteSVM::new();

        // Add programs
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        svm.add_program_from_file(
            lobbyist::id().into(),
            manifest_dir.join("../../target/deploy/lobbyist.so"),
        )
        .unwrap();
        svm.add_program_from_file(
            SQUADS_PROGRAM_ID,
            manifest_dir.join("tests/fixtures/squads_multisig.so"),
        )
        .unwrap();
        svm.add_program_from_file(
            AUTOCRAT_PROGRAM_ID,
            manifest_dir.join("tests/fixtures/autocrat.so"),
        )
        .unwrap();
        svm.add_program_from_file(
            CONDITIONAL_VAULT_PROGRAM_ID,
            manifest_dir.join("tests/fixtures/conditional_vault.so"),
        )
        .unwrap();
        svm.add_program_from_file(AMM_PROGRAM_ID, manifest_dir.join("tests/fixtures/amm.so"))
            .unwrap();

        let price_account = pubkey!("7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE");
        let price_feed_id =
            hex_to_bytes("ef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d")
                .try_into()
                .unwrap();
        let pyth_price_data = BASE64_STANDARD.decode("IvEjY51+9M1gMUcENA3t3zcf1CRyFI8kjp0abRpesqw6zYt/1dayQwHvDYtv2izrpB2hXUCV0do5Kg0vjtDGx7wPTPrIwoC1baNqsW0EAAAAe0o5AQAAAAD4////kkOfaAAAAACSQ59oAAAAAGwglXoEAAAAZ93aAAAAAAD+w3gVAAAAAAA=").unwrap();
        svm.set_account(
            price_account,
            Account {
                lamports: 1825020,
                data: pyth_price_data,
                owner: pubkey!("rec5EKMGg6MxZYaMdyBfgwp4d5rB9T1VQH5pJv5LtFJ"),
                executable: false,
                rent_epoch: 18446744073709551615,
            },
        )
        .unwrap();

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

        let usdc_mint = create_token(&mut svm, &signer, initial_supply);
        let token_mint = create_token(&mut svm, &signer, initial_supply);

        let (dao_pda, multisig_pda) = create_dao(&mut svm, &signer, token_mint, usdc_mint);

        let squads_proposal_pda = create_squads_proposal(&mut svm, &signer, multisig_pda);

        let question_pda = create_question(&mut svm, &signer, squads_proposal_pda);

        let (usdc_vault_pda, fail_usdc_mint, pass_usdc_mint) =
            create_conditional_vault(&mut svm, &signer, question_pda, usdc_mint);
        let (token_vault_pda, fail_token_mint, pass_token_mint) =
            create_conditional_vault(&mut svm, &signer, question_pda, token_mint);

        let (fail_amm_pda, fail_amm_lp_mint) =
            create_amm(&mut svm, &signer, fail_token_mint, fail_usdc_mint);
        let (pass_amm_pda, pass_amm_lp_mint) =
            create_amm(&mut svm, &signer, pass_token_mint, pass_usdc_mint);

        let (_fail_token_ata, _pass_token_ata) = split_tokens(
            &mut svm,
            &signer,
            question_pda,
            token_vault_pda,
            token_mint,
            initial_supply / 2,
            fail_token_mint,
            pass_token_mint,
        );
        let (_fail_usdc_ata, _pass_usdc_ata) = split_tokens(
            &mut svm,
            &signer,
            question_pda,
            usdc_vault_pda,
            usdc_mint,
            initial_supply / 2,
            fail_usdc_mint,
            pass_usdc_mint,
        );

        add_liquidity(
            &mut svm,
            &signer,
            fail_amm_pda,
            fail_token_mint,
            fail_usdc_mint,
            AddLiquidityArgs {
                quote_amount: initial_supply / 2,
                max_base_amount: initial_supply / 2,
                min_lp_tokens: 100,
            },
        );

        add_liquidity(
            &mut svm,
            &signer,
            pass_amm_pda,
            pass_token_mint,
            pass_usdc_mint,
            AddLiquidityArgs {
                quote_amount: initial_supply / 2,
                max_base_amount: initial_supply / 2,
                min_lp_tokens: 100,
            },
        );

        let proposal_pda = create_proposal(
            &mut svm,
            &signer,
            dao_pda,
            question_pda,
            squads_proposal_pda,
            token_vault_pda,
            usdc_vault_pda,
            fail_amm_pda,
            pass_amm_pda,
            fail_amm_lp_mint,
            pass_amm_lp_mint,
        );

        TestContext {
            svm,
            signer,
            dao: dao_pda,
            proposal: proposal_pda,
            base_mint: token_mint,
            quote_mint: usdc_mint,
            fail_amm: fail_amm_pda,
            pass_amm: pass_amm_pda,
            fail_base_mint: fail_token_mint,
            fail_quote_mint: fail_usdc_mint,
            pass_base_mint: pass_token_mint,
            pass_quote_mint: pass_usdc_mint,
            price_account,
            price_feed_id,
        }
    }
}
