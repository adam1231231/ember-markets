use std::str::FromStr;

use anchor_lang::prelude::*;
use anchor_spl::token::{Token, Mint, TokenAccount};

use crate::consts::{ADMIN_WALLETS, MARKET_AUTH_SEED};
use crate::ember_errors::EmberErr;
use crate::state::orderbook::OrderBookState;
use crate::state::state::{Auth, Market, UsersBalances};
use binary_outcome_tokens::state::Condition;

pub fn initialize_market(ctx: Context<InitializeMarket>) -> Result<()> {
    confirm_admin(&ctx.accounts.signer)?;

    {
        ctx.accounts.orderbook_state_1.load_init()?;
        ctx.accounts.orderbook_state_2.load_init()?;
        ctx.accounts.balances.load_init()?;
    }

    ctx.accounts.market.creator = *ctx.accounts.signer.key;
    ctx.accounts.market.orderbook_state_1 = ctx.accounts.orderbook_state_1.key();
    ctx.accounts.market.orderbook_state_2 = ctx.accounts.orderbook_state_2.key();

    ctx.accounts.market.balances = ctx.accounts.balances.key();
    ctx.accounts.market.resolved = false;
    Ok(())
}

pub fn initialize_vaults(ctx: Context<InitializeVaults>) -> Result<()> {

    confirm_admin(&ctx.accounts.signer)?;

    let condition_struct = &ctx.accounts.condition;

    // checking token mints match the ones in the condition
    require!(
        ctx.accounts.base_token_1.key() == condition_struct.outcomes[0].token_mint,
        EmberErr::InvalidToken
    );
    require!(
        ctx.accounts.base_token_2.key() == condition_struct.outcomes[1].token_mint,
        EmberErr::InvalidToken
    );
    require!(
        ctx.accounts.quote_token.key() == condition_struct.collateral_token,
        EmberErr::InvalidToken
    );

    ctx.accounts.market.condition_key = ctx.accounts.condition.key();
    ctx.accounts.market.quote_key = condition_struct.collateral_token.key();
    ctx.accounts.market.outcome_1_key = condition_struct.outcomes[0].token_mint;
    ctx.accounts.market.outcome_2_key = condition_struct.outcomes[1].token_mint;
    ctx.accounts.market.base_vault_1 = ctx.accounts.base_vault_1.key();
    ctx.accounts.market.base_vault_2 = ctx.accounts.base_vault_2.key();
    ctx.accounts.market.quote_vault = ctx.accounts.quote_vault.key();

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
    pub signer: Signer<'info>,

    #[account(init, payer = signer, space = Market::SIZE)]
    pub market: Box<Account<'info, Market>>,

    #[account(zero)]
    pub orderbook_state_1: AccountLoader<'info, OrderBookState>,
    #[account(zero)]
    pub orderbook_state_2: AccountLoader<'info, OrderBookState>,
    #[account(zero)]
    pub balances: AccountLoader<'info, UsersBalances>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct InitializeVaults<'info> {
    #[account(mut, constraint = market.creator == signer.key())]
    signer: Signer<'info>,

    #[account(mut)]
    pub market: Box<Account<'info, Market>>,

    #[account(mut)]
    pub condition: Box<Account<'info, Condition>>,
    #[account(init, seeds = [MARKET_AUTH_SEED, market.key().as_ref()],bump, payer = signer, space = 9)]
    pub market_auth_pda: Box<Account<'info, Auth>>,
    pub base_token_1: Box<Account<'info, Mint>>,
    pub base_token_2: Box<Account<'info, Mint>>,
    pub quote_token: Box<Account<'info, Mint>>,

    #[account(
        init,
        token::mint = base_token_1,
        token::authority = market_auth_pda,
        payer = signer)]
    pub base_vault_1: Box<Account<'info, TokenAccount>>,


    #[account(
        init,
        token::mint = base_token_2,
        token::authority = market_auth_pda,
        payer = signer)]
    pub base_vault_2: Box<Account<'info, TokenAccount>>,


    #[account(
        init,
        token::mint = quote_token,
        token::authority = market_auth_pda,
        payer = signer)]
    pub quote_vault: Box<Account<'info, TokenAccount>>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
