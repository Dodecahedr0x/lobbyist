use typhoon::prelude::*;

#[account]
pub struct Lobbyist {
    pub bump: u8,
    #[key]
    pub dao: Pubkey,
    pub token_mint: Pubkey,
    pub usdc_mint: Pubkey,
}

impl Lobbyist {
    pub const LEN: usize = 8 + std::mem::size_of::<Lobbyist>();
}
