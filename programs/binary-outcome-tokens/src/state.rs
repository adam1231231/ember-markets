use anchor_lang::prelude::*;
use anchor_lang::prelude::Pubkey;


#[account]
pub struct Condition {
    pub name: Vec<u8>, // condition name

    pub description: Vec<u8>, // condition description

    pub outcomes: [Outcome; 2], // list of outcomes, used to mint tokens and determine winner

    pub active: u64, // 1 means active, 0 means inactive and minting should be disabled.

    pub ticket_token_mint: Pubkey, // the mint of the base token / ticket token

    pub collateral_token: Pubkey, // the mint of the collateral token

    pub collateral_per_ticket: u64, // the amount of collateral tokens needed to mint one ticket

    pub resolution_auth: Pubkey, // the authority that can change the condition to inactive and start the redeem process

    pub collateral_vault: Pubkey, // the vault that holds the collateral tokens

    pub ended_at_slot: u64, // the slot at which the condition ended, this is only informative and not used in any logic
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default, Copy)]
pub struct Outcome {
    pub name: [u8; 25], // condition name, used mainly to distinguish between conditions

    pub token_mint: Pubkey, // base token mint for the condition tickets

    pub winner: u64, // defaults to 0 when created, when a condition changes to 1 then redeem starts and that outcome is the winner
}

impl Condition {
    pub const MAX_SIZE: usize = 29 + 254 + 8 + 32 + 32 + 8 + 32 + 32 + 8 + (4 + 2 * (25 + 32 * 8));
}

#[account]
#[derive(Default)]
pub struct AuthAccount {}
