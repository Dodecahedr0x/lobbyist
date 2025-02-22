use typhoon::prelude::*;
use typhoon_token::Mint;

use crate::{autocrat_cpi::Dao, state::Lobbyist};

#[context]
pub struct InitializeLobbyist {
    pub creator: Mut<Signer>,
    #[constraint(
        init,
        payer = creator,
        space = Lobbyist::SPACE,
        seeded,
        keys = [&args.admin],
    )]
    pub lobbyist: Mut<Account<Lobbyist>>,
    pub dao: BorshAccount<Dao>,
    pub token_mint: Account<Mint>,
    pub usdc_mint: Account<Mint>,
    pub system_program: Program<System>,
}

/// Creates a new lobbyist for a given decision market
pub fn initialize_lobbyist(ctx: InitializeLobbyist) -> Result<(), ProgramError> {
    *ctx.lobbyist.mut_data()? = Lobbyist {
        bump: ctx.bumps.lobbyist,
        dao: *ctx.dao.key(),
        token_mint: *ctx.token_mint.key(),
        usdc_mint: *ctx.usdc_mint.key(),
    };

    Ok(())
}
