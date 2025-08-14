mod common;

use {
    common::TestContext,
    lobbyist::*,
    solana_pubkey::Pubkey,
    solana_sdk_ids::system_program,
    solana_signer::Signer,
    solana_transaction::Transaction,
    spl_associated_token_account::{
        get_associated_token_address, instruction::create_associated_token_account_idempotent,
    },
    typhoon::lib::RefFromBytes,
    typhoon_instruction_builder::generate_instructions_client,
};

generate_instructions_client!(lobbyist);

#[test]
fn integration_test() {
    let initial_supply = 1_000_000_000;
    let mut ctx = TestContext::new(initial_supply);

    let lobbyist_pda =
        Pubkey::find_program_address(&Lobbyist::derive(&ctx.dao.to_bytes()), &lobbyist::ID.into())
            .0;
    let escrow_pda = Pubkey::find_program_address(
        &Escrow::derive(&lobbyist_pda.to_bytes(), &ctx.signer.pubkey().to_bytes()),
        &lobbyist::ID.into(),
    )
    .0;

    let lobbyist_ix = InitializeLobbyistInstruction {
        creator: ctx.signer.pubkey(),
        lobbyist: lobbyist_pda,
        dao: ctx.dao,
        proposal: ctx.proposal,
        fail_amm: ctx.fail_amm,
        pass_amm: ctx.pass_amm,
        base_mint: ctx.base_mint,
        quote_mint: ctx.quote_mint,
        system_program: system_program::id(),
    }
    .into_instruction();

    let tx = Transaction::new_signed_with_payer(
        &[lobbyist_ix],
        Some(&ctx.signer.pubkey()),
        &[&ctx.signer],
        ctx.svm.latest_blockhash(),
    );
    let result = ctx.svm.send_transaction(tx).unwrap();
    eprintln!("result: {:?}", result);

    let escrow_base_ata = get_associated_token_address(&escrow_pda, &ctx.base_mint);
    let escrow_quote_ata = get_associated_token_address(&escrow_pda, &ctx.quote_mint);
    let escrow_ix = InitializeEscrowInstruction {
        depositor: ctx.signer.pubkey(),
        lobbyist: lobbyist_pda,
        escrow: escrow_pda,
        base_mint: ctx.base_mint,
        quote_mint: ctx.quote_mint,
        escrow_base_ata,
        escrow_quote_ata,
        token_program: spl_token::ID.into(),
        ata_token_program: spl_associated_token_account::ID.into(),
        system_program: system_program::id(),
    }
    .into_instruction();

    eprintln!("escrow_ix: {:?}", escrow_ix.accounts);
    let tx = Transaction::new_signed_with_payer(
        &[escrow_ix],
        Some(&ctx.signer.pubkey()),
        &[&ctx.signer],
        ctx.svm.latest_blockhash(),
    );
    let result = ctx.svm.send_transaction(tx).unwrap();
    eprintln!("result: {:?}", result);

    let tx = Transaction::new_signed_with_payer(
        &[
            create_associated_token_account_idempotent(
                &ctx.signer.pubkey(),
                &ctx.signer.pubkey(),
                &ctx.base_mint,
                &spl_token::ID.into(),
            ),
            create_associated_token_account_idempotent(
                &ctx.signer.pubkey(),
                &ctx.signer.pubkey(),
                &ctx.quote_mint,
                &spl_token::ID.into(),
            ),
        ],
        Some(&ctx.signer.pubkey()),
        &[&ctx.signer],
        ctx.svm.latest_blockhash(),
    );
    ctx.svm.send_transaction(tx).unwrap();

    let user_base_ata = get_associated_token_address(&ctx.signer.pubkey(), &ctx.base_mint);
    let user_quote_ata = get_associated_token_address(&ctx.signer.pubkey(), &ctx.quote_mint);
    let deposit_ix = DepositInstruction {
        depositor: ctx.signer.pubkey(),
        lobbyist: lobbyist_pda,
        escrow: escrow_pda,
        base_mint: ctx.base_mint,
        quote_mint: ctx.quote_mint,
        user_base_ata,
        user_quote_ata,
        escrow_base_ata,
        escrow_quote_ata,
        token_program: spl_token::ID.into(),
        ata_token_program: spl_associated_token_account::ID.into(),
        system_program: system_program::id(),
        arg_0: DepositArgs {
            base_amount: (initial_supply / 2).into(),
            quote_amount: (initial_supply / 2).into(),
        },
    }
    .into_instruction();

    let tx = Transaction::new_signed_with_payer(
        &[deposit_ix],
        Some(&ctx.signer.pubkey()),
        &[&ctx.signer],
        ctx.svm.latest_blockhash(),
    );
    let result = ctx.svm.send_transaction(tx).unwrap();
    eprintln!("result: {:?}", result);

    let escrow_account = ctx.svm.get_account(&escrow_pda).unwrap();
    let escrow = Escrow::read(&escrow_account.data).unwrap();
    assert_eq!(escrow.base_amount, initial_supply / 2);
    assert_eq!(escrow.quote_amount, initial_supply / 2);

    let withdraw_ix = WithdrawInstruction {
        depositor: ctx.signer.pubkey(),
        lobbyist: lobbyist_pda,
        escrow: escrow_pda,
        base_mint: ctx.base_mint,
        quote_mint: ctx.quote_mint,
        user_base_ata,
        user_quote_ata,
        escrow_base_ata,
        escrow_quote_ata,
        token_program: spl_token::ID.into(),
        ata_token_program: spl_associated_token_account::ID.into(),
        system_program: system_program::id(),
        arg_0: WithdrawArgs {
            base_amount: (initial_supply / 4).into(),
            quote_amount: (initial_supply / 4).into(),
        },
    }
    .into_instruction();

    let tx = Transaction::new_signed_with_payer(
        &[withdraw_ix],
        Some(&ctx.signer.pubkey()),
        &[&ctx.signer],
        ctx.svm.latest_blockhash(),
    );
    let result = ctx.svm.send_transaction(tx).unwrap();
    eprintln!("result: {:?}", result);

    let escrow_account = ctx.svm.get_account(&escrow_pda).unwrap();
    let escrow = Escrow::read(&escrow_account.data).unwrap();
    assert_eq!(escrow.base_amount, initial_supply / 4);
    assert_eq!(escrow.quote_amount, initial_supply / 4);
}
