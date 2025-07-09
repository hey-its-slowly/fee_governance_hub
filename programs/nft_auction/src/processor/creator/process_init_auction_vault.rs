use {
    anchor_lang::prelude::*,
    anchor_spl::token::{Token, Mint},
    anchor_spl::associated_token::{AssociatedToken, create_idempotent},
};

#[derive(Accounts)]
pub struct InitAuctionVault<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: we don't read this account
    pub auction: UncheckedAccount<'info>,

    pub mint: Account<'info, Mint>,

    #[account(mut)]
    /// CHECK: we don't read this account
    pub vault: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn handler(ctx: Context<InitAuctionVault>) -> Result<()> {
    let create_idempotent_ctx = CpiContext::new(
        ctx.accounts.associated_token_program.to_account_info(),
        anchor_spl::associated_token::Create {
            payer: ctx.accounts.authority.to_account_info(),
            associated_token: ctx.accounts.vault.to_account_info(),
            authority: ctx.accounts.auction.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
        },
    );
    create_idempotent(create_idempotent_ctx)?;
    msg!("Initialized vault for mint: {}", ctx.accounts.mint.key());

    Ok(())
}