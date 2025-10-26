use {
    bytemuck::{AnyBitPattern, NoUninit},
    typhoon::prelude::*,
};

#[derive(Default, Copy, Clone, NoUninit, AnyBitPattern)]
#[repr(C)]
pub struct EscrowStatusId(u64);

impl From<EscrowStatus> for EscrowStatusId {
    fn from(status: EscrowStatus) -> Self {
        Self(status as u64)
    }
}

#[derive(Default, Copy, Clone)]
pub enum EscrowStatus {
    #[default]
    Paused,
    Active,
    Done,
}

#[derive(NoUninit, AnyBitPattern, AccountState, Copy, Clone, Debug)]
#[repr(C)]
#[no_space]
pub struct Escrow {
    pub bump: u64,
    #[key]
    pub lobbyist: Pubkey,
    #[key]
    pub depositor: Pubkey,
    pub pyth_feed_id: [u8; 32],
    pub base_amount: u64,
    pub quote_amount: u64,
    pub pass_base_amount: u64,
    pub pass_quote_amount: u64,
    pub fail_base_amount: u64,
    pub fail_quote_amount: u64,
    pub active: u8,
    pub bullish: u8,
    /// Will buy until the pass market price is this percentage of the spot price
    pub bullish_threshold_bps: i16,
    /// Will sell until the fail market price is this percentage of the spot price
    pub bearish_threshold_bps: i16,
    pub _reserved: [u8; 10],
}

impl Escrow {
    pub const LEN: usize = 8 + core::mem::size_of::<Escrow>();
}
