use {
  crate::{error::ContractError, event::*, state::*},
  anchor_lang::prelude::*,
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct RemoveRewardIx {
  reward_id: u8,
}

#[derive(Accounts)]
#[instruction(ix: RemoveRewardIx)]
pub struct RemoveRewardCtx<'info> {
  #[account(mut)]
  pub authority: Signer<'info>,

  #[account(
    mut,
    constraint = game.authority == authority.key() @ ContractError::InvalidAuthority
  )]
  pub game: Box<Account<'info, Game>>,

  pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<RemoveRewardCtx>, ix: RemoveRewardIx) -> Result<()> {
  let game = &mut ctx.accounts.game;
  let game_key = game.key();

  let reward_index = game.get_reward_token_index(ix.reward_id);
  require!(reward_index != usize::MAX, ContractError::RewardTokenNotExist);
  require!(game.reward_tokens[reward_index].num_units == 0, ContractError::RewardTokenNotEmpty);

  game.reward_tokens.remove(reward_index);

  emit!(RewardEvent {
    message: "remove_reward".to_string(),
    game: game_key,
    id: 0,
    mint: Pubkey::default(),
    decimals: 0,
    unit_value: 0,
    rarity: 0,
    num_units: 0,
  });

  Ok(())
}
