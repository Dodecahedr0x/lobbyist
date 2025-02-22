use amm::state::Amm;
use anchor_lang::prelude::*;

use crate::state::{Escrow, Lobbyist};

#[derive(Accounts)]
pub struct InitializeEscrow<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        seeds = [
            "lobbyist:".as_bytes(),
            lobbyist.proposal.key().as_ref()
        ],
        bump = lobbyist.bump,
    )]
    pub lobbyist: Account<'info, Lobbyist>,
    #[account(
        init,
        payer = owner,
        space = 8 + std::mem::size_of::<Escrow>(),
        seeds = [
            "escrow:".as_bytes(),
            lobbyist.key().as_ref(),
            owner.key().as_ref(),
        ],
        bump
    )]
    pub escrow: Account<'info, Escrow>,
    #[account(mut)]
    pub pass_amm: Account<'info, Amm>,
    #[account(mut)]
    pub fail_amm: Account<'info, Amm>,
    pub system_program: AccountInfo<'info>,
}

impl<'info> InitializeEscrow<'info> {
    pub fn handler(ctx: Context<Self>) -> Result<()> {
        ctx.accounts.escrow.set_inner(Escrow {
            bump: ctx.bumps.escrow,
            lobbyist: ctx.accounts.lobbyist.key(),
            owner: ctx.accounts.owner.key(),
            token_amount: 0,
            usdc_amount: 0,
        });

        Ok(())
    }
}
