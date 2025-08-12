use typhoon::prelude::*;

#[context]
pub struct HelloWorldContext {
    pub signer: Signer,
}

pub fn hello_world(ctx: HelloWorldContext) -> ProgramResult {
    msg!("Hello, world!");
    Ok(())
}
