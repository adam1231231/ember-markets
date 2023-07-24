use anchor_lang::prelude::*;
use anchor_spl::token;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::consts::CONDITION_AUTH_PDA_SEED;
use crate::error_codes::ErrorCodes;
use crate::state::{AuthAccount, Condition};

pub fn merge_ticket(ctx: Context<MergeTicket>, tickets_amount: u64) -> Result<()> {
    // Check that the condition is active
    if ctx.accounts.condition.active == 0 {
        return err!(ErrorCodes::ConditionInactive);
    }

    // checking their have enough tokens to merge
    if ctx.accounts.payer_1.amount < tickets_amount || ctx.accounts.payer_2.amount < tickets_amount {
        return err!(ErrorCodes::NotEnoughTokens);
    }

    // burn outcome 1 tokens from the payer wallet
    let cpi_accounts = token::Burn {
        mint: ctx.accounts.outcome_1_token.to_account_info(),
        authority: ctx.accounts.signer.to_account_info(),
        from: ctx.accounts.payer_1.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::burn(cpi_ctx, tickets_amount)?;

    // burn outcome 2 tokens from the payer wallet
    let cpi_accounts = token::Burn {
        mint: ctx.accounts.outcome_2_token.to_account_info(),
        authority: ctx.accounts.signer.to_account_info(),
        from: ctx.accounts.payer_2.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::burn(cpi_ctx, tickets_amount)?;


    // mint base tokens to the receiver wallet
    let cpi_accounts = token::MintTo {
        mint: ctx.accounts.ticket_token_mint.to_account_info(),
        to: ctx.accounts.receiver.to_account_info(),
        authority: ctx.accounts.condition_auth_pda.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();

    let bump = *ctx.bumps.get("condition_auth_pda").unwrap();
    let condition_key = ctx.accounts.condition.key();
    let seeds: &[&[&[u8]]] = &[&[
        CONDITION_AUTH_PDA_SEED.as_ref(),
        condition_key.as_ref(), &[bump]
    ]];

    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, seeds);
    token::mint_to(cpi_ctx, tickets_amount)?;


    Ok(())
}


#[derive(Accounts)]
#[instruction(tickets_amount: u64)]
pub struct MergeTicket<'info> {
    signer: Signer<'info>,

    condition: Box<Account<'info, Condition>>,

    #[account(mut, seeds = [CONDITION_AUTH_PDA_SEED, condition.key().as_ref()], bump)]
    condition_auth_pda: Account<'info, AuthAccount>,

    #[account(mut, constraint = receiver.mint.key() == condition.ticket_token_mint)]
    receiver: Box<Account<'info, TokenAccount>>,

    #[account(mut, constraint = payer_1.mint.key() == outcome_1_token.key())]
    payer_1: Box<Account<'info, TokenAccount>>,
    #[account(mut, constraint = payer_2.mint.key() == outcome_2_token.key())]
    payer_2: Box<Account<'info, TokenAccount>>,

    #[account(mut,constraint = outcome_1_token.key() == condition.outcomes[0].token_mint)]
    pub outcome_1_token: Account<'info, Mint>,

    #[account(mut, constraint = outcome_2_token.key() == condition.outcomes[1].token_mint)]
    pub outcome_2_token: Account<'info, Mint>,

    #[account(mut, constraint = ticket_token_mint.key() == condition.ticket_token_mint)]
    ticket_token_mint: Box<Account<'info, Mint>>,

    token_program: Program<'info, Token>,
}