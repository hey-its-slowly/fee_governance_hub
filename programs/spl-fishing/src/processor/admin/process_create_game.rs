use {
  crate::{constant::*, error::ContractError, event::*, state::*},
  anchor_lang::prelude::*,
  anchor_spl::token::{Mint, Token, TokenAccount},
  fee_governance_hub::{
    self,
    cpi::{transfer_fees, accounts::TransferFeesCtx},
    processor::TransferFeesIx,
    state::Config as FeeConfig,
  },
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreateGameIx {
  game_id: u64,
  payment_token_unit_value: u64,
}

#[derive(Accounts)]
#[instruction(ix: CreateGameIx)]
pub struct CreateGameCtx<'info> {
  #[account(mut)]
  pub authority: Signer<'info>,

  #[account(
    mut,
  )]
  pub colleague: Option<Box<Account<'info, Colleague>>>,

  #[account(
    init,
    seeds = [GAME_SEED, authority.key().as_ref(), ix.game_id.to_le_bytes().as_ref()],
    bump,
    payer = authority,
    space = std::mem::size_of::<Game>() + 8 + MAX_NUM_REWARD_TOKENS * std::mem::size_of::<RewardTokenInfo>() + MAX_NUM_REVEAL_PENDING_PLAYERS * 32
  )]
  pub game: Box<Account<'info, Game>>,
  
  #[account()]
  /// CHECK: we read this key only
  pub payment_token_mint: Account<'info, Mint>,

  #[account(
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

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, CreateGameCtx<'info>>, ix: CreateGameIx) -> Result<()> {
  let game = &mut ctx.accounts.game;

  if ctx.accounts.colleague.is_none() {
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
        fee_instruction_index: CREATE_GAME_INSTRUCTION_INDEX as u64,
      }
    )?;
  } else if let Some(ref mut colleague) = ctx.accounts.colleague {
    require!(colleague.wallet == ctx.accounts.authority.key(), ContractError::InvalidColleague);
    colleague.num_games += 1;
  }

  game.bump = ctx.bumps.game;
  game.authority = ctx.accounts.authority.key();  
  game.game_id = ix.game_id;

  game.payment_token_mint = ctx.accounts.payment_token_mint.key();
  game.payment_token_decimals = ctx.accounts.payment_token_mint.decimals;
  game.payment_token_unit_value = ix.payment_token_unit_value;
  game.payment_token_amount = 0;

  game.num_flips = 0;

  game.reward_tokens = Vec::new();
  game.reveal_pending_players = Vec::new();

  game.created_at = Clock::get()?.unix_timestamp as u64;

  emit!(GameEvent {
    message: "create_game".to_string(),
    authority: ctx.accounts.authority.key(),
    game: game.key(),
    game_id: ix.game_id,
    payment_token_mint: ctx.accounts.payment_token_mint.key(),
    payment_token_decimals: ctx.accounts.payment_token_mint.decimals,
    payment_token_unit_value: ix.payment_token_unit_value,
    payment_token_amount: 0,
    created_at: game.created_at,
  });

  Ok(())
}
