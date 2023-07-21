pub mod side;
pub mod orderbook;

use anchor_lang::prelude::{Pubkey, account, zero_copy};
use crate::state::orderbook::OrderBook;


#[account(zero_copy)]
pub struct Market {
    pub buy_side: OrderBook,
    pub sell_side: OrderBook,
    pub condition_key: Pubkey,
}

pub enum Sides {
    Buy,
    Sell,
}