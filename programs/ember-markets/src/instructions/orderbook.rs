use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

use crate::consts::{MARKET_AUTH_SEED, USER_ACCOUNT_PDA_SEED};
use crate::ember_errors::EmberErr;
use crate::state::orderbook::OrderBookState;
use crate::state::side::Side;
use crate::state::state::{Auth, Market, MarketSpecificUser, User, UsersBalances};
use crate::utils::{transfer_tokens, transfer_tokens_signed};

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

pub fn place_market_order(ctx: Context<PlaceMarketOrder>, side: Side, amount: u64) -> Result<()> {
    let orderbook = &mut ctx.accounts.orderbook.load_mut()?;
    let mut filled_amount = 0;
    let mut total_cost = 0;

    let balances = &mut ctx.accounts.balances.load_mut()?;
    match side {
        Side::Bid => {
            let orders_side = &mut orderbook.asks;
            let mut i = orders_side.best_order_idx;

            while filled_amount < amount && i != 0 {
                let order = orders_side.orders.get_mut(i as usize).unwrap();
                let amount_to_fill = std::cmp::min(order.size, amount - filled_amount);
                order.size -= amount_to_fill;
                total_cost += amount_to_fill * order.price;
                filled_amount += amount_to_fill;
                balances.credit_account(order.uid, amount_to_fill * order.price, 0)?;
                i = order.next;
                if order.size == 0 {
                    orders_side.remove_order(i);
                }
            }
        }
        Side::Ask => {
            let base_mint = orderbook.base_mint;
            let orders_side = &mut orderbook.bids;
            let mut i = orders_side.best_order_idx;

            while filled_amount < amount && i != 0 {
                let order = orders_side.orders.get_mut(i as usize).unwrap();
                let amount_to_fill = std::cmp::min(order.size, amount - filled_amount);
                order.size -= amount_to_fill;
                total_cost += amount_to_fill * order.price;
                filled_amount += amount_to_fill;
                if base_mint == ctx.accounts.market.outcome_1_key {
                    balances.credit_account(order.uid, amount_to_fill, 1)?;
                } else if base_mint == ctx.accounts.market.outcome_2_key {
                    balances.credit_account(order.uid, amount_to_fill, 2)?;
                } else {
                    return err!(EmberErr::InvalidMarket);
                }
                i = order.next;
                if order.size == 0 {
                    orders_side.remove_order(i);
                }
            }
        }
    }

    drop(orderbook);
    drop(balances);

    let (payer, receiver, vault_to, vault_from) = match side {
        Side::Ask => (
            &ctx.accounts.base_account,
            &ctx.accounts.quote_account,
            &ctx.accounts.base_vault,
            &ctx.accounts.quote_vault,
        ),
        Side::Bid => (
            &ctx.accounts.quote_account,
            &ctx.accounts.base_account,
            &ctx.accounts.quote_vault,
            &ctx.accounts.base_vault,
        ),
    };

    let token_program_info = &ctx.accounts.token_program.to_account_info();
    let signer = ctx.accounts.signer.to_account_info();
    // transfer the trade amount from the user to the vault
    transfer_tokens(
        signer,
        payer.to_account_info(),
        vault_to.to_account_info(),
        token_program_info.clone(),
        amount,
    )?;

    // transfer the trade amount from the vault to the user
    let bump = ctx.bumps.get("market_auth_pda").unwrap();
    let market = ctx.accounts.market.key();
    let seeds: &[&[&[u8]]] = &[&[MARKET_AUTH_SEED, market.as_ref(), &[*bump]]];

    let vault_auth = &ctx.accounts.market_auth_pda;

    transfer_tokens_signed(
        vault_auth.to_account_info(),
        vault_from.to_account_info(),
        receiver.to_account_info(),
        token_program_info.clone(),
        amount,
        seeds,
    )?;

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
    #[account(mut, constraint = balances.key() == market.balances)]
    pub balances: AccountLoader<'info, UsersBalances>,
}

#[derive(Accounts)]
pub struct PlaceMarketOrder<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    pub market: Account<'info, Market>,
    #[account(mut, seeds = [signer.key().as_ref(), USER_ACCOUNT_PDA_SEED], bump)]
    pub user_account: Account<'info, User>,
    #[account(mut, constraint = orderbook.key() == market.orderbook_state_1 || orderbook.key() == market.orderbook_state_2)]
    pub orderbook: AccountLoader<'info, OrderBookState>,
    #[account(mut, constraint = balances.key() == market.balances)]
    pub balances: AccountLoader<'info, UsersBalances>,
    #[account(mut, seeds = [MARKET_AUTH_SEED, market.key().as_ref()], bump)]
    pub market_auth_pda: Account<'info, Auth>,

    // TODO: add relevant checks
    pub base_account: Account<'info, TokenAccount>,
    pub quote_account: Account<'info, TokenAccount>,
    pub base_vault: Account<'info, TokenAccount>,
    pub quote_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}
