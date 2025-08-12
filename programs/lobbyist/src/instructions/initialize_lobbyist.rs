pub use crate::state::Lobbyist;
use {
    crate::{
        amm_cpi::{Amm, AmmProgram},
        autocrat_cpi::{AutocratProgram, Dao, Proposal},
    },
    typhoon::prelude::*,
    typhoon_token::Mint,
};

#[context]
pub struct InitializeLobbyistContext {
    pub creator: Mut<Signer>,
    #[constraint(
        init,
        payer = creator,
        space = Lobbyist::LEN,
        seeded = [&dao.key()],
        bump
    )]
    pub lobbyist: Mut<Account<Lobbyist>>,
    pub dao: BorshAccount<Dao>,
    pub fail_base_mint: Account<Mint>,
    pub fail_quote_mint: Account<Mint>,
    pub pass_base_mint: Account<Mint>,
    pub pass_quote_mint: Account<Mint>,
    pub fail_amm: BorshAccount<Amm>,
    pub pass_amm: BorshAccount<Amm>,
    pub proposal: BorshAccount<Proposal>,
    pub autocrat_program: Program<AutocratProgram>,
    pub amm_program: Program<AmmProgram>,
    pub system_program: Program<System>,
}

/// Creates a new lobbyist for a given decision market
pub fn initialize_lobbyist(ctx: InitializeLobbyistContext) -> ProgramResult {
    msg!("Initializing lobbyist");

    assert_eq!(*ctx.fail_base_mint.key(), ctx.fail_amm.data()?.base_mint);
    assert_eq!(*ctx.fail_quote_mint.key(), ctx.fail_amm.data()?.quote_mint);
    assert_eq!(*ctx.pass_base_mint.key(), ctx.pass_amm.data()?.base_mint);
    assert_eq!(*ctx.pass_quote_mint.key(), ctx.pass_amm.data()?.quote_mint);
    assert_eq!(*ctx.fail_amm.key(), ctx.proposal.data()?.fail_amm);
    assert_eq!(*ctx.pass_amm.key(), ctx.proposal.data()?.pass_amm);

    *ctx.lobbyist.mut_data()? = Lobbyist {
        bump: ctx.bumps.lobbyist,
        dao: *ctx.dao.key(),
        proposal: *ctx.proposal.key(),
        pass_amm: *ctx.pass_amm.key(),
        fail_amm: *ctx.fail_amm.key(),
        pass_base_mint: *ctx.pass_base_mint.key(),
        pass_quote_mint: *ctx.pass_quote_mint.key(),
        fail_base_mint: *ctx.fail_base_mint.key(),
        fail_quote_mint: *ctx.fail_quote_mint.key(),
    };

    Ok(())
}
