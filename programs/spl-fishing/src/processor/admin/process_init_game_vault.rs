use {
  crate::constant::*,
  anchor_lang::prelude::*,
  anchor_spl::token::{Mint, Token, TokenAccount},
};

#[derive(Accounts)]
#[instruction()]
pub struct InitGameVaultCtx<'info> {
  #[account(mut)]
  pub authority: Signer<'info>,

  /// CHECK: we read this key only
  pub game: UncheckedAccount<'info>,
  
  #[account()]
  /// CHECK: we read this key only
  pub token_mint: Account<'info, Mint>,

  #[account(
    init,
    token::mint = token_mint,
    token::authority = game,
    seeds = [VAULT_SEED, game.key().as_ref(), token_mint.key().as_ref()],
    bump,
    payer = authority,
  )]
  pub token_vault: Box<Account<'info, TokenAccount>>,

  pub token_program: Program<'info, Token>,
  pub system_program: Program<'info, System>,
}

pub fn handler(_: Context<InitGameVaultCtx>) -> Result<()> {

  Ok(())
}
