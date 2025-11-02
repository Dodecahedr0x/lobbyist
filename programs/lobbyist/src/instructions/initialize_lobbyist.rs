pub use crate::state::Lobbyist;
use {crate::futarchy_cpi::Dao, typhoon::prelude::*, typhoon_token::Mint};

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
    pub base_mint: Account<Mint>,
    pub quote_mint: Account<Mint>,
    pub system_program: Program<System>,
}

/// Creates a new lobbyist for a given decision market
pub fn initialize_lobbyist(ctx: InitializeLobbyist) -> ProgramResult {
    msg!("Initialize lobbyist");

    *ctx.lobbyist.mut_data()? = Lobbyist {
        bump: ctx.bumps.lobbyist,
        dao: *ctx.dao.key(),
        base_mint: *ctx.base_mint.key(),
        quote_mint: *ctx.quote_mint.key(),
    };

    Ok(())
}
