use typhoon::prelude::*;

#[account]
pub struct Escrow {
    pub bump: u64,
    #[key]
    pub lobbyist: Pubkey,
    #[key]
    pub owner: Pubkey,
    pub token_amount: u64,
    pub usdc_amount: u64,
}

impl Escrow {
    pub const LEN: usize = 8 + std::mem::size_of::<Escrow>();
}
