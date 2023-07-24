use anchor_lang::prelude::*;
use anchor_spl::token;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::consts::CONDITION_AUTH_PDA_SEED;
use crate::error_codes::ErrorCodes;
use crate::state::{AuthAccount, Condition};

// split one ticket into 2 outcome tokens
pub fn split_ticket(ctx: Context<SplitTicket>, tickets_amount: u64) -> Result<()> {
    // Check that the condition is active
    if ctx.accounts.condition.active == 0 {
        return err!(ErrorCodes::ConditionInactive);
    }


    // Check that the payer has enough tokens
    if ctx.accounts.payer.amount < tickets_amount {
        return err!(ErrorCodes::NotEnoughTokens);
    }

    // burn tickets from the payer wallet
    let cpi_accounts = token::Burn {
        mint: ctx.accounts.ticket_token_mint.to_account_info(),
        from: ctx.accounts.payer.to_account_info(),
        authority: ctx.accounts.signer.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::burn(cpi_ctx, tickets_amount)?;
    
    // mint 1 token for outcome 1
    let cpi_accounts = token::MintTo {
        mint: ctx.accounts.outcome_1_token.to_account_info(),
        to: ctx.accounts.receiver_1.to_account_info(),
        authority: ctx.accounts.condition_auth_pda.to_account_info(),
    };
    let condition_key = ctx.accounts.condition.key();
    let bump = *ctx.bumps.get("condition_auth_pda").unwrap();
    let seeds: &[&[&[u8]]] = &[&[
        CONDITION_AUTH_PDA_SEED.as_ref(),
        condition_key.as_ref(), &[bump]
    ]];

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, seeds);
    token::mint_to(cpi_ctx, tickets_amount)?;

    // mint 1 token for outcome 2
    let cpi_accounts = token::MintTo {
        mint: ctx.accounts.outcome_2_token.to_account_info(),
        to: ctx.accounts.receiver_2.to_account_info(),
        authority: ctx.accounts.condition_auth_pda.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, seeds);
    token::mint_to(cpi_ctx, tickets_amount)?;

    msg!("Split {} tickets successful", tickets_amount);
    Ok(())
}


#[derive(Accounts)]
#[instruction(tickets_amount: u64)]
pub struct SplitTicket<'info> {
    #[account(mut)]
    signer: Signer<'info>,

    #[account(mut)]
    condition: Box<Account<'info, Condition>>,

    #[account(mut, constraint = payer.mint.key() == condition.ticket_token_mint)]
    payer: Account<'info, TokenAccount>,

    #[account(mut, constraint = receiver_1.mint.key() == outcome_1_token.key())]
    receiver_1: Account<'info, TokenAccount>,
    #[account(mut, constraint = receiver_2.mint.key() == outcome_2_token.key())]
    receiver_2: Account<'info, TokenAccount>,

    #[account(mut, seeds = [CONDITION_AUTH_PDA_SEED, condition.key().as_ref()], bump)]
    condition_auth_pda: Account<'info, AuthAccount>,

    #[account(mut,constraint = outcome_1_token.key() == condition.outcomes[0].token_mint)]
    pub outcome_1_token: Account<'info, Mint>,

    #[account(mut,constraint = outcome_2_token.key() == condition.outcomes[1].token_mint)]
    pub outcome_2_token: Account<'info, Mint>,

    #[account(mut, constraint = ticket_token_mint.key() == condition.ticket_token_mint)]
    ticket_token_mint: Account<'info, Mint>,

    token_program: Program<'info, Token>,
}