use {
    crate::{constant::*, error::ContractError, state::*, utils::*},
    anchor_lang::prelude::*,
};

#[derive(Accounts)]
#[instruction()]
pub struct RemoveColleagueCtx<'info> {
  #[account(mut,
    constraint = is_super_admin(authority.key) @ ContractError::InvalidAuthority
  )]
  pub authority: Signer<'info>,

  #[account(
    mut,
    seeds = [COLLEAGUE_SEED, colleague_wallet.key().as_ref()],
    bump,
    close = authority,
  )]
  pub colleague: Box<Account<'info, Colleague>>,

  /// CHECK: we read this key only
  pub colleague_wallet: UncheckedAccount<'info>,

  pub system_program: Program<'info, System>,
}

pub fn handler(_: Context<RemoveColleagueCtx>) -> Result<()> {

  Ok(())
}
