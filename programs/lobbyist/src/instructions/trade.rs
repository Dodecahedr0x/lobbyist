use {
    crate::{
        errors::LobbyistError,
        price_to_u64,
        state::{Escrow, Lobbyist},
    },
    pyth_min::price_update::PriceUpdateV2,
    typhoon::prelude::*,
    typhoon_token::{AtaTokenProgram, Mint, TokenProgram},
};

pub const MAXIMUM_AGE: u64 = 10; // 10 seconds

#[context]
pub struct Trade {
    pub depositor: Mut<Signer>,
    #[constraint(
        seeded,
        bump = lobbyist.data()?.bump as u8,
        has_one = base_mint,
        has_one = quote_mint,
    )]
    pub lobbyist: Account<Lobbyist>,
    #[constraint(
        seeded,
        has_one = lobbyist,
        has_one = depositor,
        bump = escrow.data()?.bump as u8,
    )]
    pub escrow: Mut<Account<Escrow>>,
    pub pyth_price: UncheckedAccount,
    pub base_mint: Account<Mint>,
    pub quote_mint: Account<Mint>,
    pub escrow_base_ata: Mut<UncheckedAccount>,
    pub escrow_quote_ata: Mut<UncheckedAccount>,
    pub token_program: Program<TokenProgram>,
    pub ata_token_program: Program<AtaTokenProgram>,
    pub system_program: Program<System>,
}

/// Creates a new escrow for a given lobbyist
pub fn trade(ctx: Trade) -> ProgramResult {
    msg!("Trade");

    let data = &ctx.pyth_price.data()?[8..];
    let price_v2 = PriceUpdateV2::get_price_update_v2_from_bytes(data);
    let price = price_v2
        .get_price_no_older_than(
            Clock::get()?.unix_timestamp,
            MAXIMUM_AGE,
            Some(&ctx.escrow.data()?.pyth_feed_id),
        )
        .map_err(|_| LobbyistError::GetPythPrice)?;
    msg!(format!("price: {:?}", price).as_str());
    msg!(format!("price: {:?}", price_to_u64(price, 2)).as_str());

    Ok(())
}
