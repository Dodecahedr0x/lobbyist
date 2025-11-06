use {
    crate::{
        errors::LobbyistError,
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
    pub proposal: BorshAccount<Proposal>,
    #[constraint(
        seeded,
        has_one = dao @ LobbyistError::InvalidDao,
        has_one = depositor @ LobbyistError::InvalidDepositor,
        has_one = proposal @ LobbyistError::InvalidProposal,
        has_one = base_mint @ LobbyistError::InvalidBaseMint,
        has_one = quote_mint @ LobbyistError::InvalidQuoteMint,
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
            msg!("Spot");
            let spot_twap = get_twap(&spot)?;
            msg!(format!("{:?}", spot_twap).as_str());
        }
        PoolState::Futarchy { spot, pass, fail } => {
            msg!("Futarchy");
            let pass_twap = get_twap(&pass)?;
            msg!(format!("Pass TWAP: {:?}", pass_twap).as_str());
            let fail_twap = get_twap(&fail)?;
            msg!(format!("Fail TWAP: {:?}", fail_twap).as_str());
            let spot_twap = get_twap(&spot)?;
            msg!(format!("Spot TWAP: {:?}", spot_twap).as_str());
        }
    }

    Ok(())
}

fn get_twap(pool: &Pool) -> ProgramResult<u128> {
    let start_timestamp = pool.oracle.created_at_timestamp + pool.oracle.start_delay_seconds as i64;

    if pool.oracle.last_updated_timestamp <= start_timestamp {
        msg!(format!(
            "Last update too recent: {} <= {}",
            pool.oracle.last_updated_timestamp, start_timestamp,
        )
        .as_str());
        return Err(LobbyistError::LastUpdateTooRecent.into());
    }

    let seconds_passed = (pool.oracle.last_updated_timestamp - start_timestamp) as u128;

    if seconds_passed == 0 {
        msg!(format!("Not enough time passed: {}", seconds_passed).as_str());
        return Err(LobbyistError::NotEnoughTimePassed.into());
    }
    if pool.oracle.aggregator == 0 {
        msg!(format!("Invalid oracle aggregator: {}", pool.oracle.aggregator).as_str());
        return Err(LobbyistError::InvalidOracleAggregator.into());
    }

    Ok(pool.oracle.aggregator / seconds_passed)
}
