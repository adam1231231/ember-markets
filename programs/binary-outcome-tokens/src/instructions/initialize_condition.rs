use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::consts::CONDITION_AUTH_PDA_SEED;
use crate::error_codes::ErrorCodes;
use crate::state::{AuthAccount, Condition, Outcome};

fn string_to_fixed_array(s: &str) -> [u8; 25] {
    let mut result = [0u8; 25];
    let bytes = s.as_bytes();

    // Copy the bytes from the string to the fixed-size array.
    if bytes.len() <= 25 {
        result[..bytes.len()].copy_from_slice(bytes);
    } else {
        result.copy_from_slice(&bytes[..25]);
    }

    result
}


pub fn initialize_condition(
    ctx: Context<InitializeCondition>,
    name: String,
    description: String,
    outcome_1_name: String,
    outcome_2_name: String,
    collateral_per_ticket: u64,
) -> Result<()> {
    let auth_pda = ctx.accounts.condition_auth_pda.key();

    ctx.accounts.ticket_token_mint.token_check(auth_pda)?;
    ctx.accounts.outcome_token_1.token_check(auth_pda)?;
    ctx.accounts.outcome_token_2.token_check(auth_pda)?;

    ctx.accounts.condition.name = name.into_bytes();
    ctx.accounts.condition.description = description.into_bytes();
    ctx.accounts.condition.active = 1;
    ctx.accounts.condition.ticket_token_mint = ctx.accounts.ticket_token_mint.key();
    ctx.accounts.condition.outcomes = [
        Outcome {
            name: string_to_fixed_array(&outcome_1_name),
            token_mint: ctx.accounts.outcome_token_1.key().to_owned(),
            winner: 0,
        },
        Outcome {
            name: string_to_fixed_array(&outcome_2_name),
            token_mint: ctx.accounts.outcome_token_2.key().to_owned(),
            winner: 0,
        },
    ];

    ctx.accounts.condition.resolution_auth = ctx.accounts.signer.key();
    ctx.accounts.condition.collateral_token = ctx.accounts.collateral_token.key();
    ctx.accounts.condition.collateral_per_ticket = collateral_per_ticket;
    ctx.accounts.condition.collateral_vault = ctx.accounts.collateral_vault.key();
    Ok(())
}

#[derive(Accounts)]
#[instruction(name: String,
description: String,
outcome_1: String,
outcome_2: String,
collateral_per_ticket: u64)]
pub struct InitializeCondition<'info> {
    #[account(mut)]
    signer: Signer<'info>,

    #[account(init,
    payer = signer,
    space = Condition::MAX_SIZE)]
    pub condition: Box<Account<'info, Condition>>,

    #[account(init,
    seeds = [CONDITION_AUTH_PDA_SEED, condition.key().as_ref()],
    bump,
    payer = signer,
    space = 9)]
    condition_auth_pda: Account<'info, AuthAccount>,

    pub ticket_token_mint: Account<'info, Mint>,

    pub outcome_token_1: Account<'info, Mint>,

    pub outcome_token_2: Account<'info, Mint>,

    pub collateral_token: Account<'info, Mint>,

    #[account(init, payer = signer, token::mint = collateral_token, token::authority = condition_auth_pda)]
    pub collateral_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

trait TokenCheck {
    fn token_check(&self, authority : Pubkey) -> Result<()>;
}

impl TokenCheck for Mint {
    fn token_check(&self, authority : Pubkey) -> Result<()> {
        // checking if any of the tokens is pre-minted
        // avoiding the condition initializer from pre minting some token and exploiting the vault
        if self.supply > 0 {
            return err!(ErrorCodes::SupplyNotZero);
        }
        // checking if token got 0 decimals since tickets aren't dividable, avoiding complexity
        if self.decimals != 0 {
            return err!(ErrorCodes::InvalidTokenMintDecimals);
        }

        // checking if the token mint authority is held by a PDA
        if self.mint_authority.unwrap() != authority {
            return err!(ErrorCodes::InvalidTokenMintAuthority);
        }
        Ok(())
    }
}
