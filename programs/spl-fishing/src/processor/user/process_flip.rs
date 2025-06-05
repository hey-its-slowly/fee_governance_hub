use {
    crate::{constant::*, state::*, utils::*, error::ContractError, event::flip::FlipEvent},
    anchor_lang::prelude::*,
    anchor_spl::token::{Mint, Token, TokenAccount, Transfer},
    fee_governance_hub::{
      self,
      cpi::{transfer_fees, accounts::TransferFeesCtx},
      processor::TransferFeesIx,
      state::Config as FeeConfig,
    },
};

#[derive(Accounts)]
#[instruction()]
pub struct FlipCtx<'info> {
  #[account(mut)]
  pub authority: Signer<'info>,

  #[account(mut)]
  pub game: Box<Account<'info, Game>>,
  
  #[account(constraint = payment_token_mint.key() == game.payment_token_mint @ ContractError::InvalidMint)]
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

  #[account(
    constraint = fee_config.program == this_program.key() @ ContractError::InvalidFeeConfig,
  )]
  pub fee_config: Box<Account<'info, FeeConfig>>,

  /// CHECK: we read this key only
  pub fee_governance_hub: UncheckedAccount<'info>,

  /// CHECK: we read this key only
  pub this_program: UncheckedAccount<'info>,

  pub token_program: Program<'info, Token>,
  pub system_program: Program<'info, System>,
}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, FlipCtx<'info>>) -> Result<()> {
  let game = &mut ctx.accounts.game;

  require!(game.reveal_pending_players.len() < MAX_NUM_REVEAL_PENDING_PLAYERS, ContractError::ExceedMaxNumRevealPendingPlayers);

  require!(game.is_flip_available(ctx.accounts.authority.key()), ContractError::FlipNotAvailable);

  require!(ctx.accounts.this_program.key().eq(ctx.program_id), ContractError::InvalidProgramId);

  transfer_fees(
    CpiContext::new(
      ctx.accounts.fee_governance_hub.to_account_info(),
      TransferFeesCtx {
        authority: ctx.accounts.authority.to_account_info(),
        config: ctx.accounts.fee_config.to_account_info(),
        target_program: ctx.accounts.this_program.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
      }
    ).with_remaining_accounts(ctx.remaining_accounts.to_vec()),
    TransferFeesIx {
      fee_instruction_index: FLIP_INSTRUCTION_INDEX as u64,
    }
  )?;

  anchor_spl::token::transfer(
    CpiContext::new(
      ctx.accounts.token_program.to_account_info(),
      Transfer {
        from: ctx.accounts.user_payment_token_vault.to_account_info(),
        to: ctx.accounts.payment_token_vault.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
      },
    ),
    game.payment_token_unit_value,
  )?;

  game.num_flips = game.num_flips.safe_add(1).unwrap();
  game.reveal_pending_players.push(ctx.accounts.authority.key());
  game.payment_token_amount = game.payment_token_amount.safe_add(game.payment_token_unit_value).unwrap();

  emit!(FlipEvent {
    message: "flip".to_string(),
    game: game.key(),
    game_flip_index: game.num_flips,
    player: ctx.accounts.authority.key(),
    reward_id: 0,
    winning_rarity: 0,
    reward_token_mint: Pubkey::default(),
    reward_token_amount: 0,
    reward_token_decimals: 0,
  });

  Ok(())
}
