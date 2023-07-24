use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};

use crate::consts::CONDITION_AUTH_PDA_SEED;
use crate::error_codes::ErrorCodes;
use crate::state::{AuthAccount, Condition};

// burn one outcome token for the underlying collateral, only winner tokens should be accepted
pub fn redeem_payout(ctx: Context<RedeemPayout>, tickets_amount: u64) -> Result<()> {
    // if condition is still active, user can't start redeeming yet
    if ctx.accounts.condition.active == 1 {
        return err!(ErrorCodes::ConditionStillActive);
    }

    // check if enough tokens to redeem
    if ctx.accounts.payer.amount < tickets_amount {
        return err!(ErrorCodes::NotEnoughTokens);
    }

    for i in ctx.accounts.condition.outcomes.iter() {
        if i.winner == 1 {
            if i.token_mint == ctx.accounts.outcome_token.key() {
                // burn outcome tokens from the payer wallet
                let cpi_accounts = anchor_spl::token::Burn {
                    mint: ctx.accounts.outcome_token.to_account_info(),
                    authority: ctx.accounts.signer.to_account_info(),
                    from: ctx.accounts.payer.to_account_info(),
                };
                let cpi_program = ctx.accounts.token_program.to_account_info();
                let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
                anchor_spl::token::burn(cpi_ctx, tickets_amount)?;

                // send 1 collateral token to the receiver wallet
                let cpi_accounts = anchor_spl::token::Transfer {
                    from: ctx.accounts.collateral_vault.to_account_info(),
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
                anchor_spl::token::transfer(cpi_ctx, ctx.accounts.condition.collateral_per_ticket * tickets_amount)?;
                msg!("redeemed {} for {}", tickets_amount, ctx.accounts.condition.collateral_per_ticket * tickets_amount);
                return Ok(());
            }
        }
    };
    err!(ErrorCodes::OutcomeTokenNotWinner)
}


#[derive(Accounts)]
#[instruction(tickets_amount: u64)]
pub struct RedeemPayout<'info> {
    signer: Signer<'info>,

    #[account(mut)]
    condition: Box<Account<'info, Condition>>,

    #[account(mut, seeds = [CONDITION_AUTH_PDA_SEED, condition.key().as_ref()], bump)]
    condition_auth_pda: Account<'info, AuthAccount>,

    #[account(mut)]
    outcome_token: Account<'info, Mint>,

    #[account(mut, constraint = payer.mint.key() == outcome_token.key())]
    payer: Account<'info, TokenAccount>,

    #[account(mut, constraint = collateral_vault.key() == condition.collateral_vault @ ErrorCodes::InvalidCollateralVault)]
    pub collateral_vault: Account<'info, TokenAccount>,

    #[account(mut, constraint = receiver.mint.key() == condition.collateral_token)]
    receiver: Account<'info, TokenAccount>,

    token_program: Program<'info, Token>,
}