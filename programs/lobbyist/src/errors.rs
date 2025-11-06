use {
    typhoon::{
        macros::TyphoonError,
        prelude::{ProgramError, ToStr},
    },
    typhoon_errors::Error,
};

#[derive(TyphoonError)]
pub enum LobbyistError {
    // 0
    #[msg("Error: Failed to get pyth price")]
    GetPythPrice = 200,
    #[msg("Error: Invalid base mint")]
    InvalidBaseMint,
    #[msg("Error: Invalid quote mint")]
    InvalidQuoteMint,
    #[msg("Error: Invalid DAO")]
    InvalidDao,
    #[msg("Error: Invalid proposal")]
    InvalidProposal,
    // 5
    #[msg("Error: Invalid depositor")]
    InvalidDepositor,
    #[msg("Error: Last update too recent")]
    LastUpdateTooRecent,
    #[msg("Error: Not enough time passed")]
    NotEnoughTimePassed,
    #[msg("Error: Invalid oracle aggregator")]
    InvalidOracleAggregator,
}
