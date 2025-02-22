use anchor_lang::prelude::*;
use anchor_spl::token::{transfer_checked, Mint, TokenAccount, TransferChecked};

use crate::state::Escrow;

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct DepositArgs {
    pub token_amount: u64,
    pub usdc_amount: u64,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        mut,
        seeds = [
            "escrow:".as_bytes(),
            escrow.lobbyist.key().as_ref(),
            owner.key().as_ref(),
        ],
        bump = escrow.bump,
    )]
    pub escrow: Account<'info, Escrow>,
    pub token_mint: Account<'info, Mint>,
    pub usdc_mint: Account<'info, Mint>,
    #[account(
        mut,
        token::mint = token_mint,
        token::authority = owner.key(),
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        token::mint = usdc_mint,
        token::authority = owner.key(),
    )]
    pub user_usdc_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        token::mint = token_mint,
        token::authority = escrow.key(),
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        token::mint = usdc_mint,
        token::authority = escrow.key(),
    )]
    pub escrow_usdc_account: Account<'info, TokenAccount>,
    pub token_program: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
}

impl<'info> Deposit<'info> {
    pub fn handler(ctx: Context<Self>, args: DepositArgs) -> Result<()> {
        transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    mint: ctx.accounts.token_mint.to_account_info(),
                    from: ctx.accounts.user_token_account.to_account_info(),
                    to: ctx.accounts.escrow_token_account.to_account_info(),
                    authority: ctx.accounts.owner.to_account_info(),
                },
            ),
            args.token_amount,
            ctx.accounts.token_mint.decimals,
        )?;

        transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    mint: ctx.accounts.usdc_mint.to_account_info(),
                    from: ctx.accounts.user_usdc_account.to_account_info(),
                    to: ctx.accounts.escrow_usdc_account.to_account_info(),
                    authority: ctx.accounts.owner.to_account_info(),
                },
            ),
            args.usdc_amount,
            ctx.accounts.usdc_mint.decimals,
        )?;

        ctx.accounts.escrow.token_amount += args.token_amount;
        ctx.accounts.escrow.usdc_amount += args.usdc_amount;
        Ok(())
    }
}
