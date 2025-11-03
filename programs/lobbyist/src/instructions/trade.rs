use {
    crate::{
        futarchy_cpi::{Dao, Pool, PoolState, Proposal},
        state::Escrow,
    },
    typhoon::prelude::*,
    typhoon_token::{AtaTokenProgram, Mint, TokenProgram},
};

pub const MAXIMUM_AGE: u64 = 10; // 10 seconds

#[context]
pub struct Trade {
    pub depositor: Mut<Signer>,
    pub dao: BorshAccount<Dao>,
    #[constraint(
        has_one = dao,
    )]
    pub proposal: BorshAccount<Proposal>,
    #[constraint(
        seeded,
        has_one = depositor,
        has_one = proposal,
        has_one = base_mint,
        has_one = quote_mint,
        bump = escrow.data_unchecked()?.bump,
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

/// Creates a new escrow for a given lobbyist
pub fn trade(ctx: Trade, _remaining_accounts: Remaining) -> ProgramResult {
    msg!("Trade");

    match &ctx.dao.data()?.amm.state {
        PoolState::Spot { spot } => {
            let spot_twap = get_twap(&spot)?;
            msg!(format!("{:?}", spot_twap).as_str());
        }
        PoolState::Futarchy { spot, pass, fail } => {
            let spot_twap = get_twap(&spot)?;
            msg!(format!("{:?}", spot_twap).as_str());
            let pass_twap = get_twap(&pass)?;
            msg!(format!("{:?}", pass_twap).as_str());
            let fail_twap = get_twap(&fail)?;
            msg!(format!("{:?}", fail_twap).as_str());
        }
    }

    Err(ProgramError::InvalidArgument.into())
}

fn get_twap(pool: &Pool) -> ProgramResult<u128> {
    let start_timestamp = pool.oracle.created_at_timestamp + pool.oracle.start_delay_seconds as i64;

    // require_gt!(self.oracle.last_updated_timestamp, start_timestamp);
    let seconds_passed = (pool.oracle.last_updated_timestamp - start_timestamp) as u128;

    // require_neq!(seconds_passed, 0);
    // require_neq!(state.oracle.aggregator, 0);

    Ok(pool.oracle.aggregator / seconds_passed)
}
