use anchor_lang::prelude::*;

use crate::consts::CONDITION_AUTH_PDA_SEED;
use crate::error_codes::ErrorCodes;
use crate::state::{AuthAccount, Condition};

// AnnouncePayout is called by the resolution authority to announce the outcome of the condition.
pub fn resolve_condition(ctx: Context<ResolveCondition>, outcome : u64) -> Result<()> {
    if outcome > 1 {
        return err!(ErrorCodes::InvalidOutcome);
    }
    ctx.accounts.condition.active = 0;
    ctx.accounts.condition.outcomes[(outcome) as usize].winner = 1;

    ctx.accounts.condition.ended_at_slot = Clock::get()?.slot;
    Ok(())
}


#[derive(Accounts)]
#[instruction(outcome: u64)]
pub struct ResolveCondition<'info> {
    #[account(mut, constraint = signer.key() == condition.resolution_auth)]
    signer: Signer<'info>,

    #[account(mut)]
    condition: Box<Account<'info, Condition>>,

    #[account(seeds = [CONDITION_AUTH_PDA_SEED, condition.key().as_ref()], bump)]
    condition_auth_pda: Account<'info, AuthAccount>,
}