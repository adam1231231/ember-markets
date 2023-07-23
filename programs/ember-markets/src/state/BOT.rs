/// Structs from the Binary Outcome token program.
use anchor_lang::prelude::*;

#[account]
pub struct Condition {
    pub name: String,
    pub description: String,
    pub outcomes: Vec<Outcome>,
    pub active: u64,
    pub ticket_token_mint: Pubkey,
    pub collateral_token: Pubkey,
    pub collateral_per_ticket: u64,
    pub resolution_auth: Pubkey,
    pub collateral_vault: Pubkey,
    pub ended_at_slot: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Outcome {
    pub name: String,
    pub token_mint: Pubkey,
    pub winner: u64,
}
