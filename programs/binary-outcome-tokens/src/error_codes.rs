use anchor_lang::prelude::error_code;


#[error_code]
pub enum ErrorCodes{
    #[msg("Token mint decimals must be 0")]
    InvalidTokenMintDecimals,
    #[msg("condition_auth_pda must be the mint_authority of the token")]
    InvalidTokenMintAuthority,
    #[msg("Condition is not active")]
    ConditionInactive,
    #[msg("Not enough tokens")]
    NotEnoughTokens,
    #[msg("invalid collateral vault")]
    InvalidCollateralVault,
    #[msg("invalid token mint")]
    InvalidTokenMint,
    #[msg("Token supply should be 0 to initialize condition")]
    SupplyNotZero,
    #[msg("Outcome should be either 0 or 1")]
    InvalidOutcome,
    #[msg("Condition still active")]
    ConditionStillActive,
    #[msg("Outcome token is not for the winner outcome")]
    OutcomeTokenNotWinner,
    #[msg("Outcome tokens should be different")]
    SameToken,
}