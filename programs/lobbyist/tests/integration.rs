mod common;

use {
    common::TestContext,
    lobbyist::{
        amm_cpi::AmmProgram,
        autocrat_cpi::{AutocratProgram, Dao, Proposal},
        *,
    },
    solana_pubkey::Pubkey,
    solana_sdk_ids::system_program,
    solana_signer::Signer,
    solana_transaction::Transaction,
    typhoon::lib::{Owner, ProgramId},
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

    eprintln!("proposal: {:?}", ctx.proposal);
    let owner: Pubkey = Proposal::OWNER.into();
    eprintln!("proposal owner: {:?}", owner);
    eprintln!("proposal: {:?}", ctx.svm.get_account(&ctx.proposal));
    let ix = InitializeLobbyistInstruction {
        creator: ctx.signer.pubkey(),
        lobbyist: lobbyist_pda,
        dao: ctx.dao,
        proposal: ctx.proposal,
        fail_amm: ctx.fail_amm,
        pass_amm: ctx.pass_amm,
        fail_base_mint: ctx.fail_base_mint,
        fail_quote_mint: ctx.fail_quote_mint,
        pass_base_mint: ctx.pass_base_mint,
        pass_quote_mint: ctx.pass_quote_mint,
        autocrat_program: AutocratProgram::ID.into(),
        amm_program: AmmProgram::ID.into(),
        system_program: system_program::id(),
    }
    .into_instruction();
    eprintln!("lobbyist_pda: {:?}", lobbyist_pda);
    eprintln!("ix: {:?}", ix);
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&ctx.signer.pubkey()),
        &[&ctx.signer],
        ctx.svm.latest_blockhash(),
    );

    let result = ctx.svm.send_transaction(tx).unwrap();
}
