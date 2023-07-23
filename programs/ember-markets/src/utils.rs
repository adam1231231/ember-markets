use anchor_lang::prelude::*;
use anchor_spl::token;

pub fn transfer_tokens<'a>(
    authority: AccountInfo<'a>,
    from: AccountInfo<'a>,
    to: AccountInfo<'a>,
    token_program_info: AccountInfo<'a>,
    amount: u64,
) -> Result<()> {
    let cpi_accounts = token::Transfer {
        from,
        to,
        authority,
    };
    let cpi_ctx = CpiContext::new(token_program_info, cpi_accounts);
    token::transfer(cpi_ctx, amount)?;
    Ok(())
}
