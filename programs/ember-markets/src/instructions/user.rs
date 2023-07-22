use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use crate::state::state::{Balance, Market, MarketSpecificUser, User, UsersBalances};

use crate::consts::USER_ACCOUNT_PDA_SEED;

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
    let  balances = &mut ctx.accounts.balances.load_mut()?;
    let user_uid = &balances.idx + 1;
    balances.idx = user_uid;
    let user_balance = Balance {
        base: 0,
        quote: 0,
    };
    balances.users[user_uid as usize] = user_balance;
    msg!("created user with UID {}", user_uid);

    ctx.accounts.user_market_pda.uid = user_uid;
    ctx.accounts.user_market_pda.volume = 0;
    Ok(())
}

pub fn claim_balance(ctx : Context<ClaimBalance>) -> Result<()> {
    let balances = &mut ctx.accounts.balances.load_mut()?;
    let user_balance = &mut balances.users[ctx.accounts.user_market_pda.uid as usize];

    Ok(())
}



#[derive(Accounts)]
pub struct CreateUserAccount<'info> {
    #[account(mut)]
    pub signer : Signer<'info>,
    pub market : Account<'info, Market>,
    #[account(init, seeds = [signer.key().as_ref(), USER_ACCOUNT_PDA_SEED], bump, payer = signer, space = std::mem::size_of::<User>())]
    pub user_account : Account<'info, User>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub payer: Signer<'info>,
}


#[derive(Accounts)]
pub struct CreateMarketAccount<'info> {
    #[account(mut)]
    pub signer : Signer<'info>,
    pub market : Box<Account<'info, Market>>,
    pub user_account : Account<'info, User>,
    #[account(init, seeds = [signer.key().as_ref(), market.key().as_ref()], bump, payer = signer, space = std::mem::size_of::<MarketSpecificUser>())]
    pub user_market_pda : Account<'info, MarketSpecificUser>,
    pub balances : AccountLoader<'info, UsersBalances>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub payer: Signer<'info>,
}

#[derive(Accounts)]
pub struct ClaimBalance<'info> {
    #[account(mut)]
    pub signer : Signer<'info>,
    pub market : Box<Account<'info, Market>>,
    #[account(mut, seeds = [signer.key().as_ref(), market.key().as_ref()], bump)]
    pub user_market_pda : Account<'info, MarketSpecificUser>,
    pub balances : AccountLoader<'info, UsersBalances>,
    pub quoteAccount : Account<'info, TokenAccount>,
    pub baseAccount : Account<'info, TokenAccount>,
}