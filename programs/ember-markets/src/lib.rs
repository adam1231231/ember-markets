use anchor_lang::prelude::*;

use crate::instructions::*;
use crate::state::side::Side;

mod consts;
mod ember_errors;
mod instructions;
mod state;
mod utils;

declare_id!("9ERQkbvkLhxTUfw4vpRpcAtxvsUorn7HEXrGSxY5F6Zy");

#[program]
pub mod ember_markets {
    use super::*;

    pub fn initialize_market(ctx: Context<InitializeMarket>) -> Result<()> {
        instructions::initialize_market(ctx)
    }

    pub fn initialize_vaults(ctx: Context<InitializeVaults>) -> Result<()> {
        instructions::initialize_vaults(ctx)
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

    pub fn claim_balance(ctx: Context<ClaimBalance>) -> Result<()> {
        instructions::claim_balance(ctx)
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
