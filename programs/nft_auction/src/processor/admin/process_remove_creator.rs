use {
    crate::{error::ContractError, state::*, utils::*},
    anchor_lang::prelude::*,
};

#[derive(Accounts)]
pub struct RemoveCreator<'info> {
  #[account(mut,
    constraint = is_super_admin(authority.key) @ ContractError::InvalidAuthority
  )]
  pub authority: Signer<'info>,

  #[account(
    mut,
    seeds = [b"creator", creator_wallet.key().as_ref()],
    bump,
    close = authority
  )]
  pub creator: Box<Account<'info, Creator>>,

  /// CHECK: Not dangerous because only admin can send tx
  pub creator_wallet: UncheckedAccount<'info>,

  pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<RemoveCreator>) -> Result<()> {
    msg!("Removed creator: {}", ctx.accounts.creator_wallet.key());

    Ok(())
}