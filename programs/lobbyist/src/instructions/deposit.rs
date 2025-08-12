use {
    crate::state::{Escrow, Lobbyist},
    bytemuck::{AnyBitPattern, NoUninit},
    typhoon::prelude::*,
    typhoon_token::{
        spl_instructions::TransferChecked, AtaTokenProgram, Mint, TokenAccount, TokenProgram,
    },
};

#[repr(C)]
#[derive(Debug, PartialEq, AnyBitPattern, NoUninit, Copy, Clone)]
pub struct DepositArgs {
    pub token_amount: u64,
    pub usdc_amount: u64,
}

#[context]
#[args(DepositArgs)]
pub struct DepositContext {
    pub owner: Mut<Signer>,
    pub lobbyist: Account<Lobbyist>,
    #[constraint(
        seeded,
        bump = escrow.data()?.bump as u8,
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

pub fn deposit(ctx: DepositContext) -> ProgramResult {
    TransferChecked {
        from: ctx.user_token_account.as_ref(),
        mint: ctx.token_mint.as_ref(),
        to: ctx.escrow_token_account.as_ref(),
        authority: ctx.owner.as_ref(),
        amount: ctx.args.token_amount.into(),
        decimals: ctx.token_mint.data()?.decimals(),
    }
    .invoke()?;

    TransferChecked {
        from: ctx.user_usdc_account.as_ref(),
        mint: ctx.token_mint.as_ref(),
        to: ctx.escrow_usdc_account.as_ref(),
        authority: ctx.owner.as_ref(),
        amount: ctx.args.usdc_amount.into(),
        decimals: ctx.usdc_mint.data()?.decimals(),
    }
    .invoke()?;

    ctx.escrow.mut_data()?.token_amount += ctx.args.token_amount;
    ctx.escrow.mut_data()?.usdc_amount += ctx.args.usdc_amount;

    Ok(())
}
