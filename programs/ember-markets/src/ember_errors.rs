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
    QuestionTooLong,
    #[msg("Price Crosses the spread")]
    PriceCrossesTheSpread,
    #[msg("Not enough funds")]
    NotEnoughFunds,
    #[msg("Orderbook doesn't match with the market")]
    InvalidMarket,
    #[msg("The order specified belongs to a different user")]
    UnauthorizedOrderCancellation,
    #[msg("The condition account is owned by the wrong program")]
    InvalidConditionOwner,
    #[msg("Provided token doesn't match the corresponding condition token")]
    InvalidToken
}
