use bytemuck::{Pod, Zeroable};
use typhoon::prelude::*;
use typhoon_token::{
    spl_instructions::TransferChecked, AtaTokenProgram, Mint, TokenAccount, TokenProgram,
};

use crate::state::{Escrow, Lobbyist};

#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
#[repr(transparent)]
pub struct PodU64(pub [u8; 8]);

impl PodU64 {
    pub const fn from_primitive(n: u64) -> Self {
        Self(n.to_le_bytes())
    }
}
impl From<u64> for PodU64 {
    fn from(n: u64) -> Self {
        Self::from_primitive(n)
    }
}
impl From<PodU64> for u64 {
    fn from(pod: PodU64) -> Self {
        Self::from_le_bytes(pod.0)
    }
}

#[repr(C, packed)]
#[derive(Debug, PartialEq, Pod, Zeroable, Copy, Clone)]
pub struct DepositArgs {
    pub token_amount: PodU64,
    pub usdc_amount: PodU64,
}

#[context]
#[args(DepositArgs)]
pub struct DepositContext {
    pub owner: Mut<Signer>,
    pub lobbyist: Account<Lobbyist>,
    #[constraint(
        seeded,
        bump = escrow.bump,
        has_one = lobbyist,
    )]
    pub escrow: Mut<Account<Escrow>>,
    pub token_mint: Account<Mint>,
    pub usdc_mint: Account<Mint>,
    pub user_token_account: Mut<Account<TokenAccount>>,
    pub user_usdc_account: Mut<Account<TokenAccount>>,
    pub escrow_token_account: Mut<Account<TokenAccount>>,
    pub escrow_usdc_account: Mut<Account<TokenAccount>>,
    pub token_program: Program<TokenProgram>,
    pub ata_token_program: Program<AtaTokenProgram>,
}

pub fn deposit(ctx: DepositContext, args: Args<DepositArgs>) -> Result<(), ProgramError> {
    TransferChecked {
        from: ctx.user_token_account.as_ref(),
        mint: ctx.token_mint.as_ref(),
        to: ctx.escrow_token_account.as_ref(),
        authority: ctx.owner.as_ref(),
        amount: args.token_amount.into(),
        decimals: ctx.token_mint.data()?.decimals(),
    }
    .invoke()?;

    TransferChecked {
        from: ctx.user_usdc_account.as_ref(),
        mint: ctx.token_mint.as_ref(),
        to: ctx.escrow_usdc_account.as_ref(),
        authority: ctx.owner.as_ref(),
        amount: args.usdc_amount.into(),
        decimals: ctx.usdc_mint.data()?.decimals(),
    }
    .invoke()?;

    ctx.escrow.mut_data()?.token_amount += <PodU64 as Into<u64>>::into(args.token_amount);
    ctx.escrow.mut_data()?.usdc_amount += <PodU64 as Into<u64>>::into(args.usdc_amount);

    Ok(())
}
