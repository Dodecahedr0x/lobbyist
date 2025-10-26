use {
    typhoon::{
        macros::TyphoonError,
        prelude::{ProgramError, ToStr},
    },
    typhoon_errors::Error,
};

#[derive(TyphoonError)]
pub enum LobbyistError {
    #[msg("Error: Failed to get pyth price")]
    GetPythPrice = 200,
}
