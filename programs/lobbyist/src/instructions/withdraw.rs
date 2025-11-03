use {
    crate::{futarchy_cpi::Proposal, state::Escrow, PodU64},
    bytemuck::{AnyBitPattern, NoUninit},
    typhoon::prelude::*,
    typhoon_token::{
        spl_instructions::TransferChecked, AtaTokenProgram, Mint, TokenAccount, TokenProgram,
    },
};

#[derive(Debug, PartialEq, AnyBitPattern, NoUninit, Copy, Clone)]
#[repr(C)]
pub struct WithdrawArgs {
    pub base_amount: PodU64,
    pub quote_amount: PodU64,
}

#[context]
#[args(WithdrawArgs)]
pub struct Withdraw {
    pub depositor: Mut<Signer>,
    pub proposal: BorshAccount<Proposal>,
    #[constraint(
        seeded,
        bump = escrow.data_unchecked()?.bump,
        has_one = depositor,
        has_one = base_mint,
        has_one = quote_mint,
    )]
    pub escrow: Mut<Account<Escrow>>,
    pub base_mint: Account<Mint>,
    pub quote_mint: Account<Mint>,
    #[constraint(
        associated_token::mint = base_mint,
        associated_token::authority = depositor,
    )]
    pub user_base_ata: Mut<Account<TokenAccount>>,
    #[constraint(
        associated_token::mint = quote_mint,
        associated_token::authority = depositor,
    )]
    pub user_quote_ata: Mut<Account<TokenAccount>>,
    #[constraint(
        associated_token::mint = base_mint,
        associated_token::authority = escrow,
    )]
    pub escrow_base_ata: Mut<Account<TokenAccount>>,
    #[constraint(
        associated_token::mint = quote_mint,
        associated_token::authority = escrow,
    )]
    pub escrow_quote_ata: Mut<Account<TokenAccount>>,
    pub token_program: Program<TokenProgram>,
    pub ata_token_program: Program<AtaTokenProgram>,
    pub system_program: Program<System>,
}

pub fn withdraw(ctx: Withdraw) -> ProgramResult {
    msg!("Withdraw");

    let bump = [ctx.escrow.data_unchecked()?.bump as u8];
    let seeds = Escrow::derive_signer_seeds_with_bump(
        ctx.proposal.as_ref().key(),
        ctx.depositor.as_ref().key(),
        &bump,
    );

    TransferChecked {
        from: ctx.escrow_base_ata.as_ref(),
        mint: ctx.base_mint.as_ref(),
        to: ctx.user_base_ata.as_ref(),
        authority: ctx.escrow.as_ref(),
        amount: ctx.args.base_amount.into(),
        decimals: ctx.base_mint.data()?.decimals(),
    }
    .invoke_signed(&[instruction::CpiSigner::from(&seeds)])?;

    TransferChecked {
        from: ctx.escrow_quote_ata.as_ref(),
        mint: ctx.quote_mint.as_ref(),
        to: ctx.user_quote_ata.as_ref(),
        authority: ctx.escrow.as_ref(),
        amount: ctx.args.quote_amount.into(),
        decimals: ctx.quote_mint.data()?.decimals(),
    }
    .invoke_signed(&[instruction::CpiSigner::from(&seeds)])?;

    let mut escrow = ctx.escrow.mut_data()?;
    escrow.base_amount -= ctx.args.base_amount;
    escrow.quote_amount -= ctx.args.quote_amount;

    Ok(())
}
