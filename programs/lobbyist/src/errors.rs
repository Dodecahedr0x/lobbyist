use anchor_lang::prelude::*;

#[error_code]
pub enum LobbyistError {
    #[msg("Proposal is not pending")]
    InvalidProposalState,
}
