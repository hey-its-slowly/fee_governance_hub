use {
  crate::{constant::*, error::ContractError, event::*, state::*, utils::*},
  anchor_lang::prelude::*,
  anchor_spl::token::{Mint, Token, TokenAccount, Transfer},
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct WithdrawRewardIx {
  reward_id: u8,
  num_units: u64,
}

#[derive(Accounts)]
#[instruction(ix: WithdrawRewardIx)]
pub struct WithdrawRewardCtx<'info> {
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

pub fn handler(ctx: Context<WithdrawRewardCtx>, ix: WithdrawRewardIx) -> Result<()> {
  let game = &mut ctx.accounts.game;
  let game_account_info = game.to_account_info();

  let game_key = game.key();

  let reward_token_index = game.get_reward_token_index(ix.reward_id);
  require!(reward_token_index != usize::MAX, ContractError::RewardTokenNotExist);

  let game_authority_key = game.authority;
  let game_id_bytes = game.game_id.to_le_bytes();
  let game_bump = game.bump;

  let reward_token_info = &mut game.reward_tokens[reward_token_index];

  let signer_seeds = &[GAME_SEED, game_authority_key.as_ref(), game_id_bytes.as_ref(), &[game_bump]];
  let signer = &[&signer_seeds[..]];

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
    ix.num_units.safe_mul(reward_token_info.unit_value).unwrap(),
  )?;

  reward_token_info.num_units = reward_token_info.num_units.safe_sub(ix.num_units).unwrap();

  emit!(RewardEvent {
    message: "change_reward".to_string(),
    game: game_key,
    id: reward_token_info.id,
    mint: reward_token_info.mint,
    decimals: reward_token_info.decimals,
    unit_value: reward_token_info.unit_value,
    rarity: reward_token_info.rarity,
    num_units: reward_token_info.num_units,
  });

  Ok(())
}
