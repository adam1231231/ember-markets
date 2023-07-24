use anchor_lang::prelude::*;
use anchor_spl::token;
use anchor_spl::token::{Token, TokenAccount};

use crate::consts::{MARKET_AUTH_SEED, USER_ACCOUNT_PDA_SEED};
use crate::state::state::{Auth, Balance, Market, MarketSpecificUser, User, UsersBalances};
use crate::utils::transfer_tokens;

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
    let balances = &mut ctx.accounts.balances.load_mut()?;
    let user_balance = &mut balances.users[ctx.accounts.user_market_pda.uid as usize];
    let base_balance_1 = user_balance.base_1;
    let base_balance_2 = user_balance.base_2;
    // 100 is the incerement of the markets in here, so in case of the usdc market, 1 means 100 lots, or 1 cent/0.01 usdc
    let quote_balance = user_balance.quote * 100;
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

pub fn deposit_balance(
    ctx: Context<DepositBalance>,
    quote_amount: u64,
    base_1_amount: u64,
    base_2_amount: u64,
) -> Result<()> {
    let balances = &mut ctx.accounts.balances.load_mut()?;
    let user_balance = &mut balances.users[ctx.accounts.user_market_pda.uid as usize];
    user_balance.quote += quote_amount;
    user_balance.base_1 += base_1_amount;
    user_balance.base_2 += base_2_amount;

    let auth = ctx.accounts.signer.to_account_info();
    let token_account_info = ctx.accounts.quote_account.to_account_info();

    // topping up quote balance
    let quote_account = ctx.accounts.quote_account.to_account_info();
    let quote_vault = ctx.accounts.quote_vault.to_account_info();
    transfer_tokens(
        auth.clone(),
        quote_account,
        quote_vault,
        token_account_info.clone(),
        quote_amount,
    )?;

    // topping up base 1 balance
    let base_account_1 = ctx.accounts.base_account_1.to_account_info();
    let base_vault_1 = ctx.accounts.base_vault_1.to_account_info();
    transfer_tokens(
        auth.clone(),
        base_account_1,
        base_vault_1,
        token_account_info.clone(),
        base_1_amount,
    )?;

    // topping up base 2 balance
    let base_account_2 = ctx.accounts.base_account_2.to_account_info();
    let base_vault_2 = ctx.accounts.base_vault_2.to_account_info();
    transfer_tokens(
        auth.clone(),
        base_account_2,
        base_vault_2,
        token_account_info.clone(),
        base_2_amount,
    )?;

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
#[derive(Accounts)]
pub struct DepositBalance<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    pub market: Box<Account<'info, Market>>,
    #[account(mut, seeds = [signer.key().as_ref(), market.key().as_ref()], bump)]
    pub user_market_pda: Box<Account<'info, MarketSpecificUser>>,
    #[account(mut, constraint = balances.key() == market.balances)]
    pub balances: AccountLoader<'info, UsersBalances>,

    #[account(mut, constraint = base_vault_1.key() == market.base_vault_1)]
    pub base_vault_1: Box<Account<'info, TokenAccount>>,
    #[account(mut, constraint = base_vault_2.key() == market.base_vault_1)]
    pub base_vault_2: Box<Account<'info, TokenAccount>>,
    #[account(mut, constraint = quote_vault.key() == market.quote_vault)]
    pub quote_vault: Box<Account<'info, TokenAccount>>,
    #[account(mut, constraint = market.quote_key == quote_account.mint.key())]
    pub quote_account: Account<'info, TokenAccount>,
    #[account(mut, constraint = market.outcome_1_key == quote_account.mint.key())]
    pub base_account_1: Box<Account<'info, TokenAccount>>,
    #[account(mut, constraint = market.outcome_2_key == quote_account.mint.key())]
    pub base_account_2: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
}
