use {
    crate::{
        state::{Escrow, Lobbyist},
        utils::PodU64,
    },
    bytemuck::{AnyBitPattern, NoUninit},
    typhoon::prelude::*,
    typhoon_token::{spl_instructions::TransferChecked, Mint, TokenAccount, TokenProgram},
};

#[derive(Debug, PartialEq, AnyBitPattern, NoUninit, Copy, Clone)]
#[repr(C)]
pub struct DepositArgs {
    pub base_amount: PodU64,
    pub quote_amount: PodU64,
}

#[context]
#[args(DepositArgs)]
pub struct Deposit {
    pub depositor: Mut<Signer>,
    #[constraint(
        has_one = base_mint,
        has_one = quote_mint,
    )]
    pub lobbyist: Account<Lobbyist>,
    #[constraint(
        seeded,
        bump = escrow.data()?.bump as u8,
        has_one = lobbyist,
    )]
    pub escrow: Mut<Account<Escrow>>,
    pub base_mint: Account<Mint>,
    pub quote_mint: Account<Mint>,
    pub user_base_ata: Mut<Account<TokenAccount>>,
    pub user_quote_ata: Mut<Account<TokenAccount>>,
    pub escrow_base_ata: Mut<Account<TokenAccount>>,
    pub escrow_quote_ata: Mut<Account<TokenAccount>>,
    pub token_program: Program<TokenProgram>,
    pub system_program: Program<System>,
}

pub fn deposit(ctx: Deposit) -> ProgramResult {
    msg!("Deposit");

    TransferChecked {
        from: ctx.user_base_ata.as_ref(),
        mint: ctx.base_mint.as_ref(),
        to: ctx.escrow_base_ata.as_ref(),
        authority: ctx.depositor.as_ref(),
        amount: ctx.args.base_amount.into(),
        decimals: ctx.base_mint.data()?.decimals(),
    }
    .invoke()?;

    TransferChecked {
        from: ctx.user_quote_ata.as_ref(),
        mint: ctx.quote_mint.as_ref(),
        to: ctx.escrow_quote_ata.as_ref(),
        authority: ctx.depositor.as_ref(),
        amount: ctx.args.quote_amount.into(),
        decimals: ctx.quote_mint.data()?.decimals(),
    }
    .invoke()?;

    ctx.escrow.mut_data()?.base_amount += ctx.args.base_amount;
    ctx.escrow.mut_data()?.quote_amount += ctx.args.quote_amount;

    Ok(())
}
