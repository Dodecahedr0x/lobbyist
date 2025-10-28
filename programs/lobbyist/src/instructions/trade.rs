use {
    crate::state::{Escrow, Lobbyist},
    typhoon::prelude::*,
    typhoon_token::{AtaTokenProgram, Mint, TokenProgram},
};

pub const MAXIMUM_AGE: u64 = 10; // 10 seconds

#[context]
pub struct Trade {
    pub depositor: Mut<Signer>,
    #[constraint(
        seeded,
        bump = lobbyist.data_unchecked()?.bump as u8,
        has_one = base_mint,
        has_one = quote_mint,
    )]
    pub lobbyist: Account<Lobbyist>,
    #[constraint(
        seeded,
        has_one = lobbyist,
        has_one = depositor,
        bump = escrow.data_unchecked()?.bump as u8,
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
pub fn trade(_ctx: Trade, _remaining_accounts: Remaining) -> ProgramResult {
    msg!("Trade");

    Ok(())
}
