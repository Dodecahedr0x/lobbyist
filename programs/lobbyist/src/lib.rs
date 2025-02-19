use anchor_lang::prelude::*;

declare_id!("3JceRWanoEVZSqsY9UGtxPA4XsSAnSKDTNWp2Sp3QQLu");

#[program]
pub mod lobbyist {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
