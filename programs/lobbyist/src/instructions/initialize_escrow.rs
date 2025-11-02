use {
    crate::{
        futarchy_cpi::Proposal,
        state::{Escrow, Lobbyist},
        PodI16,
    },
    bytemuck::{AnyBitPattern, NoUninit},
    typhoon::prelude::*,
    typhoon_token::{ata_instructions::CreateIdempotent, AtaTokenProgram, Mint, TokenProgram},
};

#[derive(Debug, PartialEq, AnyBitPattern, NoUninit, Copy, Clone)]
#[repr(C)]
pub struct InitializeEscrowArgs {
    pub bullish_threshold_bps: PodI16,
    pub bearish_threshold_bps: PodI16,
    pub bullish: u8,
}

#[context]
#[args(InitializeEscrowArgs)]
pub struct InitializeEscrow {
    pub depositor: Mut<Signer>,
    pub proposal: BorshAccount<Proposal>,
    #[constraint(
        seeded,
        bump = lobbyist.data_unchecked()?.bump,
        has_one = base_mint,
        has_one = quote_mint,
        assert = proposal.data()?.dao == lobbyist.data()?.dao,
    )]
    pub lobbyist: Account<Lobbyist>,
    #[constraint(
        init,
        payer = depositor,
        space = Escrow::SPACE,
        seeded = [
            lobbyist.key(),
            proposal.key(),
            depositor.key(),
        ],
        bump,
    )]
    pub escrow: Mut<Account<Escrow>>,
    pub base_mint: Account<Mint>,
    pub quote_mint: Account<Mint>,
    pub escrow_base_ata: Mut<UncheckedAccount>,
    pub escrow_quote_ata: Mut<UncheckedAccount>,
    pub token_program: Program<TokenProgram>,
    pub ata_token_program: Program<AtaTokenProgram>,
    pub system_program: Program<System>,
}

pub fn initialize_escrow(ctx: InitializeEscrow) -> ProgramResult {
    msg!("Initialize escrow");

    CreateIdempotent {
        funding_account: ctx.depositor.as_ref(),
        account: ctx.escrow_base_ata.as_ref(),
        wallet: ctx.escrow.as_ref(),
        mint: ctx.base_mint.as_ref(),
        token_program: ctx.token_program.as_ref(),
        system_program: ctx.system_program.as_ref(),
    }
    .invoke()?;

    CreateIdempotent {
        funding_account: ctx.depositor.as_ref(),
        account: ctx.escrow_quote_ata.as_ref(),
        wallet: ctx.escrow.as_ref(),
        mint: ctx.quote_mint.as_ref(),
        token_program: ctx.token_program.as_ref(),
        system_program: ctx.system_program.as_ref(),
    }
    .invoke()?;

    *ctx.escrow.mut_data()? = Escrow {
        bump: ctx.bumps.escrow,
        lobbyist: *ctx.lobbyist.key(),
        proposal: *ctx.proposal.key(),
        depositor: *ctx.depositor.key(),
        active: false.into(),
        bullish: ctx.args.bullish,
        base_amount: 0,
        quote_amount: 0,
        pass_base_amount: 0,
        pass_quote_amount: 0,
        fail_base_amount: 0,
        fail_quote_amount: 0,
        bullish_threshold_bps: 0,
        bearish_threshold_bps: 0,
        _reserved: [0; 9],
    };

    Ok(())
}
