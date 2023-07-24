use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint};
use anchor_spl::token::{Token, TokenAccount};

use crate::consts::CONDITION_AUTH_PDA_SEED;
use crate::error_codes::ErrorCodes;
use crate::state::{AuthAccount, Condition};

// would redeem a full ticket for it's underlying collateral
pub fn redeem_ticket(ctx: Context<RedeemTicket>, tickets_amount: u64) -> Result<()> {

    // Check that the condition is active
    if ctx.accounts.condition.active == 0 {
        return err!(ErrorCodes::ConditionInactive);
    }

    // Check that the payer has enough tokens
    if ctx.accounts.payer.amount < tickets_amount {
        return err!(ErrorCodes::NotEnoughTokens);
    }


    // burn the tickets
    let cpi_accounts = token::Burn {
        mint: ctx.accounts.ticket_token_mint.to_account_info(),
        from: ctx.accounts.payer.to_account_info(),
        authority: ctx.accounts.signer.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::burn(cpi_ctx, tickets_amount)?;

    // transfer from collateral wallet to the receiver
    let condition_key = ctx.accounts.condition.key();
    let bump = *ctx.bumps.get("condition_auth_pda").unwrap();
    let seeds: &[&[&[u8]]] = &[&[
        CONDITION_AUTH_PDA_SEED.as_ref(),
        condition_key.as_ref(), &[bump]
    ]];

    let cpi_accounts = token::Transfer {
        from: ctx.accounts.collateral_vault.to_account_info(),
        to: ctx.accounts.receiver.to_account_info(),
        authority: ctx.accounts.condition_auth_pda.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, seeds);
    let refunded_amount = tickets_amount * ctx.accounts.condition.collateral_per_ticket;
    token::transfer(cpi_ctx, refunded_amount)?;

    msg!("burnt {} tickets for {}", tickets_amount, refunded_amount);

    Ok(())
}

#[derive(Accounts)]
#[instruction(tickets_amount: u64)]
pub struct RedeemTicket<'info> {
    signer: Signer<'info>,

    #[account(mut)]
    condition: Box<Account<'info, Condition>>,

    #[account(mut, constraint = payer.mint.key() == condition.ticket_token_mint)]
    payer: Account<'info, TokenAccount>,

    #[account(mut, constraint = receiver.mint.key() == condition.collateral_token)]
    receiver: Account<'info, TokenAccount>,

    #[account(mut, constraint = ticket_token_mint.key() == condition.ticket_token_mint)]
    ticket_token_mint: Account<'info, Mint>,

    #[account(mut, seeds = [CONDITION_AUTH_PDA_SEED, condition.key().as_ref()], bump)]
    condition_auth_pda: Account<'info, AuthAccount>,

    #[account(mut, constraint = collateral_vault.key() == condition.collateral_vault @ ErrorCodes::InvalidCollateralVault)]
    pub collateral_vault: Account<'info, TokenAccount>,

    token_program: Program<'info, Token>,

}