use {
    crate::state::{Escrow, Lobbyist},
    typhoon::prelude::*,
    typhoon_token::{ata_instructions::CreateIdempotent, AtaTokenProgram, Mint, TokenProgram},
};

#[context]
pub struct InitializeEscrowContext {
    pub depositor: Mut<Signer>,
    #[constraint(
        seeded,
        has_one = base_mint,
        has_one = quote_mint,
    )]
    pub lobbyist: Account<Lobbyist>,
    #[constraint(
        init,
        payer = depositor,
        space = Escrow::LEN,
        seeded = [
            lobbyist.key(),
            depositor.key(),
        ],
        bump,
    )]
    pub escrow: Mut<Account<Escrow>>,
    pub base_mint: Account<Mint>,
    pub quote_mint: Account<Mint>,
    pub escrow_base_ata: Mut<UncheckedAccount>,
    pub escrow_quote_ata: Mut<UncheckedAccount>,
    pub token_program: Program<TokenProgram>,
    pub ata_token_program: Program<AtaTokenProgram>,
    pub system_program: Program<System>,
}

/// Creates a new escrow for a given lobbyist
pub fn initialize_escrow(ctx: InitializeEscrowContext) -> ProgramResult {
    msg!("Initialize escrow");

    CreateIdempotent {
        funding_account: ctx.depositor.as_ref(),
        account: ctx.escrow_base_ata.as_ref(),
        wallet: ctx.escrow.as_ref(),
        mint: ctx.base_mint.as_ref(),
        token_program: ctx.token_program.as_ref(),
        system_program: ctx.system_program.as_ref(),
    }
    .invoke()?;

    CreateIdempotent {
        funding_account: ctx.depositor.as_ref(),
        account: ctx.escrow_quote_ata.as_ref(),
        wallet: ctx.escrow.as_ref(),
        mint: ctx.quote_mint.as_ref(),
        token_program: ctx.token_program.as_ref(),
        system_program: ctx.system_program.as_ref(),
    }
    .invoke()?;

    *ctx.escrow.mut_data()? = Escrow {
        bump: ctx.bumps.escrow as u64,
        lobbyist: *ctx.lobbyist.key(),
        depositor: *ctx.depositor.key(),
        base_amount: 0,
        quote_amount: 0,
        pass_base_amount: 0,
        pass_quote_amount: 0,
        fail_base_amount: 0,
        fail_quote_amount: 0,
    };

    Ok(())
}
