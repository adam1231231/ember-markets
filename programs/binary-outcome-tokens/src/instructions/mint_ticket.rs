use anchor_lang::prelude::*;
use anchor_spl::token;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::consts::CONDITION_AUTH_PDA_SEED;
use crate::error_codes::ErrorCodes;
use crate::state::{AuthAccount, Condition};


// would deposit collateral to the condition and mint a ticket
pub fn mint_ticket(ctx: Context<MintTicket>, tickets_amount: u64) -> Result<()> {

    // Check that the condition is active
    if ctx.accounts.condition.active == 0 {
        return err!(ErrorCodes::ConditionInactive);
    }

    let tickets_cost = ctx.accounts.condition.collateral_per_ticket * tickets_amount;

    // Check that the payer has enough tokens
    if ctx.accounts.payer.amount < tickets_cost {
        return err!(ErrorCodes::NotEnoughTokens);
    }

    // transfer tokens to the vault
    let cpi_accounts = token::Transfer {
        from: ctx.accounts.payer.to_account_info(),
        to: ctx.accounts.collateral_vault.to_account_info(),
        authority: ctx.accounts.signer.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    token::transfer(cpi_ctx, ctx.accounts.condition.collateral_per_ticket * tickets_amount)?;
    msg!("transferred {} to the vault", tickets_cost);

    // mint tickets to the receiver wallet
    let condition_key = ctx.accounts.condition.key();
    let cpi_accounts = token::MintTo {
        mint: ctx.accounts.ticket_token_mint.to_account_info(),
        to: ctx.accounts.receiver.to_account_info(),
        authority: ctx.accounts.condition_auth_pda.to_account_info(),
    };
    let bump = *ctx.bumps.get("condition_auth_pda").unwrap();
    let seeds: &[&[&[u8]]] = &[&[
        CONDITION_AUTH_PDA_SEED.as_ref(),
        condition_key.as_ref(), &[bump]
    ]];

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, seeds);
    token::mint_to(cpi_ctx, tickets_amount)?;

    msg!("minted {} tickets to the receiver", tickets_amount);
    Ok(())
}


#[derive(Accounts)]
#[instruction(tickets_amount: u64)]
pub struct MintTicket<'info> {
    signer: Signer<'info>,

    #[account(mut)]
    condition: Box<Account<'info, Condition>>,

    #[account(mut, constraint = payer.mint.key() == condition.collateral_token)]
    payer: Account<'info, TokenAccount>,

    #[account(mut, constraint = receiver.mint.key() == condition.ticket_token_mint)]
    receiver: Account<'info, TokenAccount>,

    #[account(mut, seeds = [CONDITION_AUTH_PDA_SEED, condition.key().as_ref()], bump)]
    condition_auth_pda: Account<'info, AuthAccount>,

    #[account(mut, constraint = collateral_vault.key() == condition.collateral_vault @ ErrorCodes::InvalidCollateralVault)]
    pub collateral_vault: Account<'info, TokenAccount>,

    #[account(mut, constraint = ticket_token_mint.key() == condition.ticket_token_mint @ ErrorCodes::InvalidTokenMint)]
    pub ticket_token_mint: Account<'info, Mint>,

    token_program: Program<'info, Token>,
}
