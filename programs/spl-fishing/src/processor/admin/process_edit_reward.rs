use {
  crate::{constant::*, error::ContractError, event::*, state::*, utils::*},
  anchor_lang::prelude::*,
  anchor_spl::token::{Mint, Token, TokenAccount, Transfer},
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct EditRewardIx {
  reward_id: u8,
  new_unit_value: u64,
  new_rarity: u8,
}

#[derive(Accounts)]
#[instruction(ix: EditRewardIx)]
pub struct EditRewardCtx<'info> {
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
    mut,
    token::mint = reward_token_mint,
    token::authority = authority,
  )]
  pub user_reward_token_vault: Box<Account<'info, TokenAccount>>,

  #[account(
    mut,
    token::mint = reward_token_mint,
    token::authority = game,
    seeds = [VAULT_SEED, game.key().as_ref(), reward_token_mint.key().as_ref()],
    bump,
  )]
  pub reward_token_vault: Box<Account<'info, TokenAccount>>,

  pub token_program: Program<'info, Token>,
  pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<EditRewardCtx>, ix: EditRewardIx) -> Result<()> {
  let game = &mut ctx.accounts.game;
  let game_account_info = game.to_account_info();

  let game_key = game.key();

  let reward_token_index = game.get_reward_token_index(ix.reward_id);
  require!(reward_token_index != usize::MAX, ContractError::RewardTokenNotExist);

  let game_authority_key = game.authority;
  let game_id_bytes = game.game_id.to_le_bytes();
  let game_bump = game.bump;
  let has_new_rarity = game.has_rarity(ix.new_rarity);

  let reward_token_info = &mut game.reward_tokens[reward_token_index];

  let current_rarity = reward_token_info.rarity;
  if ix.new_rarity > 1 && ix.new_rarity != current_rarity { // if not "common" and new rarity is different from current rarity
    require!(!has_new_rarity, ContractError::RarityAlreadyExist);
  }

  let signer_seeds = &[GAME_SEED, game_authority_key.as_ref(), game_id_bytes.as_ref(), &[game_bump]];
  let signer = &[&signer_seeds[..]];

  if reward_token_info.unit_value < ix.new_unit_value {
    anchor_spl::token::transfer(
      CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
          from: ctx.accounts.user_reward_token_vault.to_account_info(),
          to: ctx.accounts.reward_token_vault.to_account_info(),
          authority: ctx.accounts.authority.to_account_info(),
        },
      ),
      (ix.new_unit_value.safe_sub(reward_token_info.unit_value).unwrap()).safe_mul(reward_token_info.num_units).unwrap(),
    )?;
  } else if reward_token_info.unit_value > ix.new_unit_value {
    anchor_spl::token::transfer(
      CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
          from: ctx.accounts.reward_token_vault.to_account_info(),
          to: ctx.accounts.user_reward_token_vault.to_account_info(),
          authority: game_account_info,
        },
        signer,
      ),
      (reward_token_info.unit_value.safe_sub(ix.new_unit_value).unwrap()).safe_mul(reward_token_info.num_units).unwrap(),
    )?;
  }

  reward_token_info.unit_value = ix.new_unit_value;
  reward_token_info.rarity = ix.new_rarity;

  emit!(RewardEvent {
    message: "change_reward".to_string(),
    game: game_key,
    id: reward_token_info.id,
    mint: reward_token_info.mint,
    decimals: reward_token_info.decimals,
    unit_value: ix.new_unit_value,
    rarity: ix.new_rarity,
    num_units: reward_token_info.num_units,
  });

  Ok(())
}
