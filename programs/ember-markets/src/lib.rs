use anchor_lang::prelude::*;

use crate::instructions::*;

mod consts;
mod ember_errors;
mod instructions;
mod state;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod ember_markets {
    use super::*;

    pub fn initialize_market(
        ctx: Context<InitializeMarket>,
        question: String,
        duration: u64,
        rewards_multiplier: u64,
    ) -> Result<()> {
        instructions::initialize_market(ctx, question, duration, rewards_multiplier)
    }

    pub fn create_user_account(ctx: Context<CreateUserAccount>) -> Result<()> {
        instructions::create_user_account(ctx)
    }

    pub fn create_market_account(ctx: Context<CreateMarketAccount>) -> Result<()> {
        instructions::create_market_account(ctx)
    }
}
