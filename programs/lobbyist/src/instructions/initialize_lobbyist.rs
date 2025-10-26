pub use crate::state::Lobbyist;
use {
    crate::autocrat_cpi::{Dao, Proposal},
    typhoon::prelude::*,
};

#[context]
pub struct InitializeLobbyist {
    pub creator: Mut<Signer>,
    #[constraint(
        init,
        payer = creator,
        space = Lobbyist::SPACE,
        seeded = [&dao.key()],
        bump
    )]
    pub lobbyist: Mut<Account<Lobbyist>>,
    #[constraint(
        has_one = base_mint,
        has_one = quote_mint,
    )]
    pub dao: BorshAccount<Dao>,
    pub base_mint: UncheckedAccount,
    pub quote_mint: UncheckedAccount,
    #[constraint(
        has_one = dao,
        has_one = fail_amm,
        has_one = pass_amm,
    )]
    pub proposal: BorshAccount<Proposal>,
    pub fail_amm: UncheckedAccount,
    pub pass_amm: UncheckedAccount,
    pub system_program: Program<System>,
}

/// Creates a new lobbyist for a given decision market
pub fn initialize_lobbyist(ctx: InitializeLobbyist) -> ProgramResult {
    msg!("Initialize lobbyist");

    // Check mints
    assert_eq!(*ctx.base_mint.key(), ctx.dao.data()?.base_mint);
    assert_eq!(*ctx.quote_mint.key(), ctx.dao.data()?.quote_mint);

    // Check proposal
    assert_eq!(*ctx.dao.key(), ctx.proposal.data()?.dao);
    assert_eq!(*ctx.fail_amm.key(), ctx.proposal.data()?.fail_amm);
    assert_eq!(*ctx.pass_amm.key(), ctx.proposal.data()?.pass_amm);

    // Check amms
    assert_eq!(*ctx.fail_amm.key(), ctx.proposal.data()?.fail_amm);
    assert_eq!(*ctx.pass_amm.key(), ctx.proposal.data()?.pass_amm);

    *ctx.lobbyist.mut_data()? = Lobbyist {
        bump: ctx.bumps.lobbyist,
        dao: *ctx.dao.key(),
        proposal: *ctx.proposal.key(),
        pass_amm: *ctx.pass_amm.key(),
        fail_amm: *ctx.fail_amm.key(),
        base_mint: *ctx.base_mint.key(),
        quote_mint: *ctx.quote_mint.key(),
    };

    Ok(())
}
