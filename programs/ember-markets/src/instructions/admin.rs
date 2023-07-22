use std::str::FromStr;

use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

use crate::consts::{ADMIN_WALLETS, BINARY_OUTCOME_TOKEN_PROGRAM_ID};
use crate::ember_errors::EmberErr;
use crate::state::BOT::Condition;
use crate::state::orderbook::OrderBookState;
use crate::state::state::{Market, UsersBalances};

pub fn initialize_market(ctx: Context<InitializeMarket>,
                         question: String,
                         duration: u64,
                         rewards_multiplier: u64) -> Result<()> {
    confirm_admin(&ctx.accounts.signer)?;

    // initially going to be 8 hours, will remove this constraint later
    if duration < 60 * 60 * 8 {
        return err!(EmberErr::DurationTooShort);
    }

    // forcing it to be one or more to avoid some complexity,
    // each 100 means 1, not using floats in here
    if rewards_multiplier < 100 {
        return err!(EmberErr::RewardsMultiplierTooSmall);
    }

    if question.len() > 200 {
        return err!(EmberErr::QuestionTooLong);
    }

    {
        ctx.accounts.orderbook_state.load_init()?;
        ctx.accounts.balances.load_init()?;
    }

    ctx.accounts.market.question = question.into_bytes();
    ctx.accounts.market.end_time = Clock::get()?.epoch + duration;
    ctx.accounts.market.creator = *ctx.accounts.signer.key;
    ctx.accounts.market.orderbook_state = ctx.accounts.orderbook_state.key();
    ctx.accounts.market.rewards_multiplier = rewards_multiplier;
    ctx.accounts.market.balances_address = ctx.accounts.balances.key();
    ctx.accounts.market.resolved = false;
    ctx.accounts.market.condition_key = ctx.accounts.condition.key();
    ctx.accounts.market.quote_key = ctx.accounts.condition.collateral_token.key();
    ctx.accounts.market.outcome_1_key = ctx.accounts.condition.outcomes[0].token_mint;
    ctx.accounts.market.outcome_2_key = ctx.accounts.condition.outcomes[1].token_mint;

    Ok(())
}


pub fn confirm_admin(signer_address: &Signer) -> Result<()> {
    let market_admin_addresses: Vec<Pubkey> = ADMIN_WALLETS
        .iter()
        .map(|address| Pubkey::from_str(address).unwrap())
        .collect();

    if !market_admin_addresses.contains(&signer_address.key()) {
        return Err(EmberErr::InvalidAdmin.into());
    }
    Ok(())
}


#[derive(Accounts)]
pub struct InitializeMarket<'info> {
    #[account(mut)]
    signer: Signer<'info>,
    #[account(init, payer = signer, space = std::mem::size_of::< Market > ())]
    market: Box<Account<'info, Market>>,
    #[account(mut)]
    orderbook_state: AccountLoader<'info, OrderBookState>,
    #[account(mut)]
    balances: AccountLoader<'info, UsersBalances>,
    #[account(owner = BINARY_OUTCOME_TOKEN_PROGRAM_ID)]
    condition: Account<'info, Condition>,
    system_program: Program<'info, System>,

}