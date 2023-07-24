use anchor_lang::prelude::*;

use crate::instructions::*;
use crate::state::side::Side;

mod consts;
mod ember_errors;
mod instructions;
mod state;
mod utils;

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

    pub fn place_limit_order(
        ctx: Context<PlaceLimitOrder>,
        side: Side,
        price: u64,
        size: u64,
        expire_in: u64,
    ) -> Result<()> {
        instructions::place_limit_order(ctx, side, price, size, expire_in)
    }

    pub fn cancel_limit_order(
        ctx: Context<CancelLimitOrder>,
        side: Side,
        order_id: u64,
    ) -> Result<()> {
        instructions::cancel_limit_order(ctx, side, order_id)
    }

    pub fn place_market_order(
        ctx: Context<PlaceMarketOrder>,
        side: Side,
        amount: u64,
    ) -> Result<()> {
        instructions::place_market_order(ctx, side, amount)
    }

    pub fn deposit_balance(
        ctx: Context<DepositBalance>,
        quote_amount: u64,
        base_1_amount: u64,
        base_2_amount: u64,
    ) -> Result<()> {
        instructions::deposit_balance(ctx, quote_amount, base_1_amount, base_2_amount)
    }

    pub fn clear_expired_orders(ctx: Context<ClearExpiredOrders>) -> Result<()> {
        instructions::clear_expired_orders(ctx)
    }
}
