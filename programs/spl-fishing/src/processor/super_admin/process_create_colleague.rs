use {
    crate::{constant::*, error::ContractError, state::*, utils::*},
    anchor_lang::prelude::*,
};

#[derive(Accounts)]
#[instruction()]
pub struct CreateColleagueCtx<'info> {
  #[account(mut,
    constraint = is_super_admin(authority.key) @ ContractError::InvalidAuthority
  )]
  pub authority: Signer<'info>,

  #[account(
    init,
    seeds = [COLLEAGUE_SEED, colleague_wallet.key().as_ref()],
    bump,
    payer = authority,
    space = std::mem::size_of::<Colleague>() + 8,
  )]
  pub colleague: Box<Account<'info, Colleague>>,

  /// CHECK: we read this key only
  pub colleague_wallet: UncheckedAccount<'info>,

  pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<CreateColleagueCtx>) -> Result<()> {
  ctx.accounts.colleague.bump = ctx.bumps.colleague;
  ctx.accounts.colleague.wallet = ctx.accounts.colleague_wallet.key();
  ctx.accounts.colleague.num_games = 0;
  
  ctx.accounts.colleague.created_at = Clock::get()?.unix_timestamp as u64;

  Ok(())
}
