use bytemuck::{AnyBitPattern, NoUninit};
use typhoon::prelude::*;

#[derive(NoUninit, AnyBitPattern, AccountState, Copy, Clone)]
#[repr(C)]
#[no_space]
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
    pub const LEN: usize = 8 + core::mem::size_of::<Escrow>();
}
