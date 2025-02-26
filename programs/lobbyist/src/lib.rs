//! Lobbyists manage funds of participants in decision markets and trade according to their targets.
use typhoon::prelude::*;

// mod errors;
mod instructions;
mod state;

use instructions::*;
use typhoon::macros::program_id;

program_id!("3JceRWanoEVZSqsY9UGtxPA4XsSAnSKDTNWp2Sp3QQLu");

anchor_cpi!("idls/autocrat.json");
// anchor_cpi!("idls/amm.json");

handlers! {
    initialize_lobbyist,
    initialize_escrow,
    deposit,
}

// /// Deposits tokens into an escrow
// pub fn deposit(ctx: Context<Deposit>, args: DepositArgs) -> Result<(), ProgramError> {
//     Deposit::handler(ctx, args)
// }
