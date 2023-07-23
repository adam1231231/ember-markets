use anchor_lang::prelude::*;
use anchor_spl::token;
use anchor_spl::token::{Token, TokenAccount};

use crate::consts::{MARKET_AUTH_SEED, USER_ACCOUNT_PDA_SEED};
use crate::state::state::{Auth, Balance, Market, MarketSpecificUser, User, UsersBalances};

pub fn create_user_account(ctx: Context<CreateUserAccount>) -> Result<()> {
    ctx.accounts.user_account.volume = 0;
    ctx.accounts.user_account.owner = ctx.accounts.signer.key();
    ctx.accounts.user_account.idle = false;
    ctx.accounts.user_account.has_open_orders = false;
    ctx.accounts.user_account.losing_bets = 0;
    ctx.accounts.user_account.winning_bets = 0;
    Ok(())
}

pub fn create_market_account(ctx: Context<CreateMarketAccount>) -> Result<()> {
    let balances = &mut ctx.accounts.balances.load_mut()?;
    let user_uid = &balances.idx + 1;
    balances.idx = user_uid;
    let user_balance = Balance {
        base_1: 0,
        base_2: 0,
        quote: 0,
    };
    balances.users[user_uid as usize] = user_balance;
    msg!("created user with UID {}", user_uid);

    ctx.accounts.user_market_pda.uid = user_uid;
    ctx.accounts.user_market_pda.volume = 0;
    Ok(())
}

pub fn claim_balance(ctx: Context<ClaimBalance>) -> Result<()> {
    // TODO: sort out having 2 different markets, 1 market for each outcome so 2 base and 1 quote account
    let balances = &mut ctx.accounts.balances.load_mut()?;
    let user_balance = &mut balances.users[ctx.accounts.user_market_pda.uid as usize];
    let base_balance_1 = user_balance.base_1;
    let base_balance_2 = user_balance.base_2;
    let quote_balance = user_balance.quote;
    user_balance.base_1 = 0;
    user_balance.base_2 = 0;
    user_balance.quote = 0;

    let bump = ctx.bumps.get("market_auth_pda").unwrap();
    let market = &ctx.accounts.market.key();

    let seeds: &[&[&[u8]]] = &[&[MARKET_AUTH_SEED, market.as_ref(), &[*bump]]];

    // transfer outcome 1 token balance
    let cpi_accounts = token::Transfer {
        from: ctx.accounts.base_vault_1.to_account_info(),
        to: ctx.accounts.base_account_1.to_account_info(),
        authority: ctx.accounts.market_auth_pda.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, seeds);
    token::transfer(cpi_ctx, base_balance_1)?;

    // transfer outcome 1 token balance
    let cpi_accounts = token::Transfer {
        from: ctx.accounts.base_vault_2.to_account_info(),
        to: ctx.accounts.base_account_2.to_account_info(),
        authority: ctx.accounts.market_auth_pda.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, seeds);
    token::transfer(cpi_ctx, base_balance_2)?;

    // transfer quote balance
    let cpi_accounts = token::Transfer {
        from: ctx.accounts.quote_vault.to_account_info(),
        to: ctx.accounts.quote_account.to_account_info(),
        authority: ctx.accounts.market_auth_pda.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, seeds);
    token::transfer(cpi_ctx, quote_balance)?;

    msg!(
        "claimed balances, outcome token 1: {}, outcome token 2: {}, quote: {} ",
        base_balance_1,
        base_balance_2,
        quote_balance
    );

    Ok(())
}

#[derive(Accounts)]
pub struct CreateUserAccount<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    pub market: Account<'info, Market>,
    #[account(init, seeds = [signer.key().as_ref(), USER_ACCOUNT_PDA_SEED], bump, payer = signer, space = std::mem::size_of::< User > ())]
    pub user_account: Account<'info, User>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateMarketAccount<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    pub market: Box<Account<'info, Market>>,
    pub user_account: Account<'info, User>,
    #[account(init, seeds = [signer.key().as_ref(), market.key().as_ref()], bump, payer = signer, space = std::mem::size_of::< MarketSpecificUser > ())]
    pub user_market_pda: Account<'info, MarketSpecificUser>,
    pub balances: AccountLoader<'info, UsersBalances>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub payer: Signer<'info>,
}

#[derive(Accounts)]
pub struct ClaimBalance<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    pub market: Box<Account<'info, Market>>,
    #[account(mut, seeds = [signer.key().as_ref(), market.key().as_ref()], bump)]
    pub user_market_pda: Account<'info, MarketSpecificUser>,
    #[account(mut, constraint = balances.key() == market.balances)]
    pub balances: AccountLoader<'info, UsersBalances>,
    #[account(mut, seeds = [MARKET_AUTH_SEED, market.key().as_ref()], bump)]
    pub market_auth_pda: Account<'info, Auth>,
    #[account(mut, constraint = base_vault_1.key() == market.base_vault_1)]
    pub base_vault_1: Account<'info, TokenAccount>,
    #[account(mut, constraint = base_vault_2.key() == market.base_vault_1)]
    pub base_vault_2: Account<'info, TokenAccount>,
    #[account(mut, constraint = quote_vault.key() == market.quote_vault)]
    pub quote_vault: Account<'info, TokenAccount>,
    #[account(mut, constraint = market.quote_key == quote_account.mint.key())]
    pub quote_account: Account<'info, TokenAccount>,
    #[account(mut, constraint = market.outcome_1_key == quote_account.mint.key())]
    pub base_account_1: Account<'info, TokenAccount>,
    #[account(mut, constraint = market.outcome_2_key == quote_account.mint.key())]
    pub base_account_2: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}
