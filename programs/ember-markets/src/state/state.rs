use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;

use crate::consts::USERS_BALANCES;
use crate::ember_errors::EmberErr;
use crate::state::side::Side;

#[account]
pub struct Market {
    pub question: Vec<u8>,
    pub condition_key: Pubkey,
    pub rewards_multiplier: u64,
    pub end_time: u64,
    pub creator: Pubkey,
    pub orderbook_state_1: Pubkey,
    pub orderbook_state_2: Pubkey,
    pub balances: Pubkey,
    pub resolved: bool,
    pub quote_key: Pubkey,
    pub outcome_1_key: Pubkey,
    pub outcome_2_key: Pubkey,
    pub base_vault_1: Pubkey,
    pub base_vault_2: Pubkey,
    pub quote_vault: Pubkey,
}

impl Market {
    pub fn confirm_base_account(&self, orderbook: Pubkey, base_account: Pubkey) -> bool {
        if orderbook == self.orderbook_state_1 {
            if base_account == self.base_vault_1 {
                return true;
            }
            false
        } else if orderbook == self.orderbook_state_2 {
            if base_account == self.base_vault_2 {
                return true;
            }
            false
        } else {
            false
        }
    }
}

#[account(zero_copy)]
pub struct UsersBalances {
    pub idx: u64,
    pub users: [Balance; USERS_BALANCES],
}

impl UsersBalances {
    pub fn debt_account(
        &mut self,
        uid: u64,
        amount: u64,
        side: Side,
        token: Option<u8>,
    ) -> Result<()> {
        let token = token.unwrap_or(0);
        match side {
            Side::Bid => {
                self.users[uid as usize].quote = self.users[uid as usize]
                    .quote
                    .checked_sub(amount)
                    .ok_or(EmberErr::NotEnoughFunds)?
            }
            Side::Ask => {
                if token == 1 {
                    self.users[uid as usize].base_1 = self.users[uid as usize]
                        .base_1
                        .checked_sub(amount)
                        .ok_or(EmberErr::NotEnoughFunds)?
                } else if token == 2 {
                    self.users[uid as usize].base_2 = self.users[uid as usize]
                        .base_2
                        .checked_sub(amount)
                        .ok_or(EmberErr::NotEnoughFunds)?
                }
            }
        }

        Ok(())
    }
}

#[zero_copy]
pub struct Balance {
    pub quote: u64,
    pub base_1: u64,
    pub base_2: u64,
}

#[account]
pub struct User {
    pub owner: Pubkey,
    pub idle: bool,
    pub has_open_orders: bool,
    pub volume: u64,
    pub winning_bets: u64,
    pub losing_bets: u64,
}

#[account]
pub struct MarketSpecificUser {
    pub uid: u64,
    pub volume: u64,
    // add avg buy in, would be nice to calculate user's pnl on without fetching transactions
}

#[account]
pub struct Auth {}
