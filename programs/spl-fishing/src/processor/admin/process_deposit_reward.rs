use {
  crate::{constant::*, error::ContractError, event::*, state::*, utils::*},
  anchor_lang::prelude::*,
  anchor_spl::token::{Mint, Token, TokenAccount, Transfer},
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct DepositRewardIx {
  reward_id: u8,
  num_units: u64,
}

#[derive(Accounts)]
#[instruction(ix: DepositRewardIx)]
pub struct DepositRewardCtx<'info> {
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

pub fn handler(ctx: Context<DepositRewardCtx>, ix: DepositRewardIx) -> Result<()> {
  let game = &mut ctx.accounts.game;
  let game_key = game.key();

  let reward_token_index = game.get_reward_token_index(ix.reward_id);
  require!(reward_token_index != usize::MAX, ContractError::RewardTokenNotExist);

  let reward_token_info = &mut game.reward_tokens[reward_token_index];

  anchor_spl::token::transfer(
    CpiContext::new(
      ctx.accounts.token_program.to_account_info(),
      Transfer {
        from: ctx.accounts.user_reward_token_vault.to_account_info(),
        to: ctx.accounts.reward_token_vault.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
      },
    ),
    ix.num_units.safe_mul(reward_token_info.unit_value).unwrap(),
  )?;

  reward_token_info.num_units = reward_token_info.num_units.safe_add(ix.num_units).unwrap();

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
