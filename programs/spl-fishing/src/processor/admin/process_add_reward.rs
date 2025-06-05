use {
  crate::{constant::*, error::ContractError, event::*, state::*},
  anchor_lang::prelude::*,
  anchor_spl::token::{Mint, Token, TokenAccount},
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct AddRewardIx {
  unit_value: u64,
  rarity: u8,
}

#[derive(Accounts)]
#[instruction(ix: AddRewardIx)]
pub struct AddRewardCtx<'info> {
  #[account(mut)]
  pub authority: Signer<'info>,

  #[account(
    mut,
    constraint = game.authority == authority.key() @ ContractError::InvalidAuthority
  )]
  pub game: Box<Account<'info, Game>>,
  
  #[account()]
  /// CHECK: we read this key only
  pub reward_token_mint: Account<'info, Mint>,

  #[account(
    token::mint = reward_token_mint,
    token::authority = game,
    seeds = [VAULT_SEED, game.key().as_ref(), reward_token_mint.key().as_ref()],
    bump,
  )]
  pub reward_token_vault: Box<Account<'info, TokenAccount>>,

  pub token_program: Program<'info, Token>,
  pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<AddRewardCtx>, ix: AddRewardIx) -> Result<()> {
  let game = &mut ctx.accounts.game;

  require!(game.reward_tokens.len() < MAX_NUM_REWARD_TOKENS, ContractError::ExceedMaxNumRewardTokens);

  if ix.rarity > 1 { // if not "common"
    require!(!game.has_rarity(ix.rarity), ContractError::RarityAlreadyExist);
  }

  let mut reward_id = game.reward_tokens.len() as u8 + 1;
  if reward_id > 1 {
    reward_id = game.reward_tokens[reward_id as usize - 2].id + 1;
  }

  game.reward_tokens.push(RewardTokenInfo {
    id: reward_id,
    mint: ctx.accounts.reward_token_mint.key(),
    unit_value: ix.unit_value,
    decimals: ctx.accounts.reward_token_mint.decimals,
    rarity: ix.rarity,
    num_units: 0,
    num_winners: 0,
  });

  emit!(RewardEvent {
    message: "add_reward".to_string(),
    game: game.key(),
    id: reward_id,
    mint: ctx.accounts.reward_token_mint.key(),
    decimals: ctx.accounts.reward_token_mint.decimals,
    unit_value: ix.unit_value,
    rarity: ix.rarity,
    num_units: 0,
  });

  Ok(())
}
