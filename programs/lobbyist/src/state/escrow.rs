use {
    bytemuck::{AnyBitPattern, NoUninit},
    typhoon::prelude::*,
};

#[derive(NoUninit, AnyBitPattern, AccountState, Copy, Clone)]
#[repr(C)]
#[no_space]
pub struct Escrow {
    pub bump: u64,
    #[key]
    pub lobbyist: Pubkey,
    #[key]
    pub depositor: Pubkey,
    pub base_amount: u64,
    pub quote_amount: u64,
    pub pass_base_amount: u64,
    pub pass_quote_amount: u64,
    pub fail_base_amount: u64,
    pub fail_quote_amount: u64,
}

impl Escrow {
    pub const LEN: usize = 8 + core::mem::size_of::<Escrow>();
}
