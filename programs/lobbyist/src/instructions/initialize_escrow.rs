use typhoon::prelude::*;

use crate::state::{Escrow, Lobbyist};

#[context]
pub struct InitializeEscrowContext {
    pub owner: Mut<Signer>,
    #[constraint(seeded)]
    pub lobbyist: Account<Lobbyist>,
    #[constraint(
        init,
        payer = owner,
        space = Escrow::LEN,
        seeded,
        keys = [
            lobbyist.key(),
            owner.key(),
        ],
    )]
    pub escrow: Mut<Account<Escrow>>,
    pub system_program: Program<System>,
}

// /// Creates a new escrow for a given lobbyist
pub fn initialize_escrow(ctx: InitializeEscrowContext) -> Result<(), ProgramError> {
    *ctx.escrow.mut_data()? = Escrow {
        bump: ctx.bumps.escrow as u64,
        lobbyist: *ctx.lobbyist.key(),
        owner: *ctx.owner.key(),
        token_amount: 0,
        usdc_amount: 0,
    };

    Ok(())
}
