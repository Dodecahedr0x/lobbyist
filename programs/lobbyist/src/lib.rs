//! Lobbyists manage funds of participants in decision markets and trade according to their targets.
use typhoon::prelude::*;

mod errors;
mod instructions;
mod state;
mod utils;

use typhoon::macros::program_id;

pub use {instructions::*, state::*, utils::*};

program_id!("3JceRWanoEVZSqsY9UGtxPA4XsSAnSKDTNWp2Sp3QQLu");

anchor_cpi!("idls/autocrat.json");
anchor_cpi!("idls/amm.json");
anchor_cpi!("idls/conditional_vault.json");

handlers! {
    initialize_lobbyist,
    initialize_escrow,
    deposit,
    withdraw,
    trade,
}
