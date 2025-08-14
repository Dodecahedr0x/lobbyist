use {
    bytemuck::{AnyBitPattern, NoUninit},
    typhoon::prelude::*,
};

#[derive(NoUninit, AnyBitPattern, AccountState, Copy, Clone)]
#[repr(C)]
#[no_space]
pub struct Lobbyist {
    pub bump: u8,
    #[key]
    pub dao: Pubkey,
    pub proposal: Pubkey,
    pub pass_amm: Pubkey,
    pub fail_amm: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
}

impl Lobbyist {
    pub const LEN: usize = 8 + core::mem::size_of::<Lobbyist>();
}
