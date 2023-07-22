use anchor_lang::prelude::*;
use crate::consts::USERS_BALANCES;

#[account]
pub struct Market {
    pub question : Vec<u8>,
    pub condition_key: Pubkey,
    pub rewards_multiplier: u64,
    pub end_time : u64,
    pub creator: Pubkey,
    pub orderbook_state : Pubkey,
    pub balances_address : Pubkey,
    pub resolved : bool,
    pub quote_key : Pubkey,
    pub outcome_1_key : Pubkey,
    pub outcome_2_key : Pubkey,
}

#[account(zero_copy)]
pub struct UsersBalances {
    pub idx : u64,
    pub users: [Balance; USERS_BALANCES],
}

#[zero_copy]
pub struct Balance {
    pub quote: u64,
    pub base: u64,
}

#[account]
pub struct User {
    pub owner: Pubkey,
    pub idle: bool,
    pub has_open_orders: bool,
    pub volume: u64,
    pub winning_bets : u64,
    pub losing_bets : u64,
}

#[account]
pub struct MarketSpecificUser {
    pub uid : u64,
    pub volume: u64,
    // add avg buy in, would be nice to calculate user's pnl on without fetching transactions
}
