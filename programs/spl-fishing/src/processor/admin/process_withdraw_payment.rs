use {
  crate::{constant::*, error::ContractError, event::*, state::*},
  anchor_lang::prelude::*,
  anchor_spl::token::{Mint, Token, TokenAccount, Transfer},
};

#[derive(Accounts)]
#[instruction()]
pub struct WithdrawPaymentCtx<'info> {
  #[account(mut)]
  pub authority: Signer<'info>,

  #[account(
    mut,
    constraint = game.authority == authority.key() @ ContractError::InvalidAuthority,
    constraint = game.payment_token_mint == payment_token_mint.key() @ ContractError::InvalidPaymentTokenMint
  )]
  pub game: Box<Account<'info, Game>>,
  
  #[account()]
  /// CHECK: we read this key only
  pub payment_token_mint: Account<'info, Mint>,

  #[account(
    mut,
    token::mint = payment_token_mint,
    token::authority = authority,
  )]
  pub user_payment_token_vault: Box<Account<'info, TokenAccount>>,

  #[account(
    mut,
    token::mint = payment_token_mint,
    token::authority = game,
    seeds = [VAULT_SEED, game.key().as_ref(), payment_token_mint.key().as_ref()],
    bump,
  )]
  pub payment_token_vault: Box<Account<'info, TokenAccount>>,

  pub token_program: Program<'info, Token>,
  pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<WithdrawPaymentCtx>) -> Result<()> {
  let game = &mut ctx.accounts.game;
  let game_account_info = game.to_account_info();

  let game_authority_key = game.authority;
  let game_id_bytes = game.game_id.to_le_bytes();
  let game_bump = game.bump;

  let signer_seeds = &[GAME_SEED, game_authority_key.as_ref(), game_id_bytes.as_ref(), &[game_bump]];
  let signer = &[&signer_seeds[..]];

  anchor_spl::token::transfer(
    CpiContext::new_with_signer(
      ctx.accounts.token_program.to_account_info(),
      Transfer {
        from: ctx.accounts.payment_token_vault.to_account_info(),
        to: ctx.accounts.user_payment_token_vault.to_account_info(),
        authority: game_account_info,
      },
      signer,
    ),
    game.payment_token_amount,
  )?;

  game.payment_token_amount = 0;

  emit!(GameEvent {
    message: "change_game".to_string(),
    authority: ctx.accounts.authority.key(),
    game: game.key(),
    game_id: game.game_id,
    payment_token_mint: game.payment_token_mint,
    payment_token_decimals: game.payment_token_decimals,
    payment_token_unit_value: game.payment_token_unit_value,
    payment_token_amount: 0,
    created_at: game.created_at,
  });

  Ok(())
}
