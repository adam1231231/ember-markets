use anchor_lang::prelude::*;

#[error_code]
pub enum EmberErr {
    #[msg("Orderbook is full and price is inferior to all other orders")]
    OrderBookFull,
    #[msg("The duration of the market is too short")]
    DurationTooShort,
    #[msg("The rewards multiplier is too small, should be 100 or more")]
    RewardsMultiplierTooSmall,
    #[msg("The signer is not an admin ")]
    InvalidAdmin,
    #[msg("Question should be 200 letters or less")]
    QuestionTooLong
}