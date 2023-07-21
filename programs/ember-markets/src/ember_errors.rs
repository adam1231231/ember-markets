use anchor_lang::prelude::*;

#[error_code]
pub enum EmberErr {
    #[msg("Orderbook is full and price is inferior to all other orders")]
    OrderbookFull,
}