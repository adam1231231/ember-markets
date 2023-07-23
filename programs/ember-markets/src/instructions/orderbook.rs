use anchor_lang::prelude::*;

use crate::ember_errors::EmberErr;
use crate::state::orderbook::OrderBookState;
use crate::state::side::Side;
use crate::state::state::{Market, MarketSpecificUser, UsersBalances};

pub fn place_limit_order(
    ctx: Context<PlaceLimitOrder>,
    side: Side,
    price: u64,
    size: u64,
) -> Result<()> {
    let orderbook = &mut ctx.accounts.orderbook.load_mut()?;
    // check other side for better price:
    // TODO: should be improved to enable fills if prices crosses the spread, but a feature for another day.
    match side {
        Side::Bid => {
            let best_price = orderbook.asks.orders[orderbook.asks.best_order_idx as usize].price;
            if price > best_price {
                return err!(EmberErr::PriceCrossesTheSpread);
            }
        }
        Side::Ask => {
            let best_price = orderbook.bids.orders[orderbook.bids.best_order_idx as usize].price;
            if price < best_price {
                return err!(EmberErr::PriceCrossesTheSpread);
            }
        }
    }

    let uid = ctx.accounts.user_market_pda.uid;
    // market is going to be outcome token / usdc denominated, so no need to implement limit orders.
    // with price, each 1 is worth is 0.01 cents (100 usdc lots), and the base token got 0 decimals so not divisible
    let total_cost = match side {
        Side::Bid => {
            orderbook.bids.insert_order(size, price, uid)?;
            price * size
        }
        Side::Ask => {
            orderbook.asks.insert_order(size, price, uid)?;
            size
        }
    };

    let market = &ctx.accounts.market;
    let balances = &mut ctx.accounts.balances.load_mut()?;
    match side {
        Side::Bid => {
            balances.debt_account(uid, total_cost, 0)?;
        }
        Side::Ask => {
            if orderbook.base_mint == market.outcome_1_key {
                balances.debt_account(uid, size, 1)?;
            } else if orderbook.base_mint == market.outcome_2_key {
                balances.debt_account(uid, size, 2)?;
            } else {
                return err!(EmberErr::InvalidMarket);
            }
        }
    }

    Ok(())
}

pub fn cancel_limit_order(
    ctx: Context<CancelLimitOrder>,
    side: Side,
    order_idx: u64,
) -> Result<()> {
    let orderbook = &mut ctx.accounts.orderbook.load_mut()?;
    let balances = &mut ctx.accounts.balances.load_mut()?;
    let orderbook_base_mint = orderbook.base_mint;

    let uid = ctx.accounts.user_market_pda.uid;
    match side {
        Side::Bid => {
            let order = &mut orderbook.bids.orders[order_idx as usize];
            if order.uid != uid {
                return err!(EmberErr::UnauthorizedOrderCancellation);
            }
            balances.credit_account(uid, order.size * order.price, 0)?;
            orderbook.bids.remove_order(order_idx);
        }
        Side::Ask => {
            let order = &mut orderbook.asks.orders[order_idx as usize];
            if order.uid != uid {
                return err!(EmberErr::UnauthorizedOrderCancellation);
            }
            if orderbook_base_mint == ctx.accounts.market.outcome_1_key {
                balances.credit_account(uid, order.size, 1)?;
            } else if orderbook_base_mint == ctx.accounts.market.outcome_2_key {
                balances.credit_account(uid, order.size, 2)?;
            } else {
                return err!(EmberErr::InvalidMarket);
            }
            orderbook.asks.remove_order(order_idx);
        }
    }

    Ok(())
}

#[derive(Accounts)]
pub struct CancelLimitOrder<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    pub market: Account<'info, Market>,
    // each market got 2 orderbooks, one for each outcome token, ctx accepts any of them
    #[account(mut, constraint = orderbook.key() == market.orderbook_state_1 || orderbook.key() == market.orderbook_state_2)]
    pub orderbook: AccountLoader<'info, OrderBookState>,

    #[account(mut, seeds = [signer.key().as_ref(), market.key().as_ref()], bump)]
    pub user_market_pda: Account<'info, MarketSpecificUser>,

    pub balances: AccountLoader<'info, UsersBalances>,
}

#[derive(Accounts)]
pub struct PlaceLimitOrder<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    pub market: Account<'info, Market>,
    // each market got 2 orderbooks, one for each outcome token, ctx accepts any of them
    #[account(mut, constraint = orderbook.key() == market.orderbook_state_1 || orderbook.key() == market.orderbook_state_2)]
    pub orderbook: AccountLoader<'info, OrderBookState>,

    #[account(mut, seeds = [signer.key().as_ref(), market.key().as_ref()], bump)]
    pub user_market_pda: Account<'info, MarketSpecificUser>,

    pub balances: AccountLoader<'info, UsersBalances>,
}
