use std::str::FromStr;
use anchor_lang::prelude::Pubkey;
use anchor_lang::solana_program;
use anchor_lang::solana_program::pubkey;

pub const ORDER_BOOK_SIZE: usize = 128;

pub const USERS_BALANCES: usize = 1000;


pub const ADMIN_WALLETS: &'static [&str; 1] = &["5GrCgeZRNtGgKe7ezhSo5vU6ug68JsrC1FCo9246DBgg"];

pub const USER_ACCOUNT_PDA_SEED: &[u8] = b"user_account_pda_seed";

pub const BINARY_OUTCOME_TOKEN_PROGRAM_ID : Pubkey = Pubkey::from_str("BinaRYoUTCoMEToKeN111111111111111111111111111").unwrap();