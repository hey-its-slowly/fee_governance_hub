use {
  crate::{constant::*, error::ContractError, state::*, utils::*, event::flip::FlipEvent},
  anchor_lang::prelude::*,
  anchor_spl::token::{Mint, Token, TokenAccount, Transfer},
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct SendPayoutIx {
  game_flip_index: u64,
  reward_id: u8,
  rarity: u8, // if 0, no reward token will be sent
}

#[derive(Accounts)]
#[instruction(ix: SendPayoutIx)]
pub struct SendPayoutCtx<'info> {
  #[account(mut,
    constraint = is_super_admin(authority.key) @ ContractError::InvalidAuthority
  )]
  pub authority: Signer<'info>,

  /// CHECK: we read this key only
  pub player: UncheckedAccount<'info>,

  #[account(mut)]
  pub game: Box<Account<'info, Game>>,
  
  #[account()]
  /// CHECK: we read this key only
  pub reward_token_mint: Account<'info, Mint>,

  #[account(
    mut,
    token::mint = reward_token_mint,
    token::authority = player,
  )]
  pub user_reward_token_vault: Option<Box<Account<'info, TokenAccount>>>,

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

pub fn handler(ctx: Context<SendPayoutCtx>, ix: SendPayoutIx) -> Result<()> {
  let game = &mut ctx.accounts.game;
  let game_account_info = game.to_account_info();
  let game_account_key = game_account_info.key();

  if ix.rarity > 0 {
    let game_authority_key = game.authority;
    let game_id_bytes = game.game_id.to_le_bytes();
    let game_bump = game.bump;

    let reward_index = game.get_reward_token_index(ix.reward_id);
    require!(reward_index != usize::MAX, ContractError::RewardTokenNotExist);

    let reward_token_info = &mut game.reward_tokens[reward_index];
    
    require!(reward_token_info.num_units > 0, ContractError::NoRewardTokenLeft);
    require!(reward_token_info.mint.eq(&ctx.accounts.reward_token_mint.key()), ContractError::RewardTokenMismatch);

    let signer_seeds = &[GAME_SEED, game_authority_key.as_ref(), game_id_bytes.as_ref(), &[game_bump]];
    let signer = &[&signer_seeds[..]];

    anchor_spl::token::transfer(
      CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
          from: ctx.accounts.reward_token_vault.to_account_info(),
          to: ctx.accounts.user_reward_token_vault.as_ref().unwrap().to_account_info(),
          authority: game_account_info,
        },
        signer,
      ),
      reward_token_info.unit_value
    )?;

    reward_token_info.num_units = reward_token_info.num_units.safe_sub(1).unwrap();
    reward_token_info.num_winners = reward_token_info.num_winners.safe_add(1).unwrap();

    emit!(FlipEvent {
      message: "send_payout".to_string(),
      game: game_account_key,
      game_flip_index: ix.game_flip_index,
      player: ctx.accounts.player.key(),
      reward_id: ix.reward_id,
      winning_rarity: ix.rarity,
      reward_token_mint: ctx.accounts.reward_token_mint.key(),
      reward_token_amount: reward_token_info.unit_value,
      reward_token_decimals: reward_token_info.decimals,
    });
  } else {
    emit!(FlipEvent {
      message: "send_payout".to_string(),
      game: game_account_key,
      game_flip_index: ix.game_flip_index,
      player: ctx.accounts.player.key(),
      reward_id: 0,
      winning_rarity: 0,
      reward_token_mint: Pubkey::default(),
      reward_token_amount: 0,
      reward_token_decimals: 0,
    });
  }

  let player_index = game.reveal_pending_players.iter().position(|&x| x == ctx.accounts.player.key());
  if let Some(index) = player_index {
    game.reveal_pending_players.remove(index);
  } else {
    return Err(ContractError::PlayerNotInPendingList.into());
  }

  Ok(())
}
