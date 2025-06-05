use {
  crate::{error::ContractError, event::*, state::*},
  anchor_lang::prelude::*,
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct EditGameIx {
  payment_token_unit_value: u64,
}

#[derive(Accounts)]
#[instruction(ix: EditGameIx)]
pub struct EditGameCtx<'info> {
  #[account(mut)]
  pub authority: Signer<'info>,

  #[account(
    mut,
    constraint = game.authority == authority.key() @ ContractError::InvalidAuthority
  )]
  pub game: Box<Account<'info, Game>>,

  pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<EditGameCtx>, ix: EditGameIx) -> Result<()> {
  let game = &mut ctx.accounts.game;

  game.payment_token_unit_value = ix.payment_token_unit_value;

  emit!(GameEvent {
    message: "change_game".to_string(),
    authority: ctx.accounts.authority.key(),
    game: game.key(),
    game_id: game.game_id,
    payment_token_mint: game.payment_token_mint,
    payment_token_decimals: game.payment_token_decimals,
    payment_token_unit_value: ix.payment_token_unit_value,
    payment_token_amount: game.payment_token_amount,
    created_at: game.created_at,
  });

  Ok(())
}
