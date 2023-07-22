mod instructions;
mod ember_errors;
mod state;
mod consts;


use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod ember_markets {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    signer : Signer<'info>,
}
