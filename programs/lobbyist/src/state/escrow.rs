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
pub struct Escrow {
    /// The lobbyist than can use this escrow
    #[key]
    pub lobbyist: Pubkey,
    /// The owner of the escrow
    #[key]
    pub depositor: Pubkey,
    /// The price feed used to trade
    pub pyth_feed_id: [u8; 32],
    /// Amount of base token owned by the escrow
    pub base_amount: u64,
    /// Amount of quote token owned by the escrow
    pub quote_amount: u64,
    /// Amount of pass base token owned by the escrow
    pub pass_base_amount: u64,
    /// Amount of pass quote token owned by the escrow
    pub pass_quote_amount: u64,
    /// Amount of fail base token owned by the escrow
    pub fail_base_amount: u64,
    /// Amount of fail quote token owned by the escrow
    pub fail_quote_amount: u64,
    /// Whether this escrow can trade
    pub active: u8,
    /// The trading preference of the escrow
    pub bullish: u8,
    /// Will buy until the pass market price is this percentage of the spot price
    pub bullish_threshold_bps: i16,
    /// Will sell until the fail market price is this percentage of the spot price
    pub bearish_threshold_bps: i16,
    /// The canonical bump
    pub bump: u8,
    pub _reserved: [u8; 9],
}
