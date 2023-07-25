use anchor_lang::prelude::*;

pub use instructions::*;

mod consts;
mod error_codes;
mod instructions;
pub mod state;

declare_id!("5c5A6f6HQNhgaSmwuKCkCcgEJWk9UoskR9S2Fp5ig6v1");

#[program]
pub mod binary_outcome_tokens {
    use super::*;

    pub fn initialize_condition(
        ctx: Context<InitializeCondition>,
        name: String,
        description: String,
        outcome_1_name: String,
        outcome_2_name: String,
        collateral_per_ticket: u64,
    ) -> Result<()> {
        instructions::initialize_condition(
            ctx,
            name,
            description,
            outcome_1_name,
            outcome_2_name,
            collateral_per_ticket,
        )
    }

    pub fn mint_ticket(ctx: Context<MintTicket>, tickets_amount: u64) -> Result<()> {
        instructions::mint_ticket(ctx, tickets_amount)
    }

    pub fn redeem_ticket(ctx: Context<RedeemTicket>, tickets_amount: u64) -> Result<()> {
        instructions::redeem_ticket(ctx, tickets_amount)
    }

    pub fn split_ticket(ctx: Context<SplitTicket>, tickets_amount: u64) -> Result<()> {
        instructions::split_ticket(ctx, tickets_amount)
    }

    pub fn merge_ticket(ctx: Context<MergeTicket>, tickets_amount: u64) -> Result<()> {
        instructions::merge_ticket(ctx, tickets_amount)
    }

    pub fn resolve_condition(ctx: Context<ResolveCondition>, outcome : u64) -> Result<()> {
        instructions::resolve_condition(ctx, outcome)
    }

    pub fn redeem_payout(ctx: Context<RedeemPayout>, tickets_amount : u64) -> Result<()> {
        instructions::redeem_payout(ctx,tickets_amount)
    }
}

