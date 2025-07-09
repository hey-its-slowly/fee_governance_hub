use {
    crate::{error::ContractError, state::*, utils::*},
    anchor_lang::prelude::*,
};

#[derive(Accounts)]
pub struct AddCreator<'info> {
  #[account(mut,
    constraint = is_super_admin(authority.key) @ ContractError::InvalidAuthority
  )]
  pub authority: Signer<'info>,

  #[account(
    init,
    seeds = [b"creator", creator_wallet.key().as_ref()],
    bump,
    payer = authority,
    space = std::mem::size_of::<Creator>() + 8,
  )]
  pub creator: Box<Account<'info, Creator>>,

  /// CHECK: Not dangerous because only admin can send tx
  pub creator_wallet: UncheckedAccount<'info>,

  /// CHECK: Not dangerous because only admin can send tx
  pub fee_wallet: UncheckedAccount<'info>,

  pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<AddCreator>, fee_type: u8, fee_amount: u64) -> Result<()> {
    ctx.accounts.creator.bump = ctx.bumps.creator;
    ctx.accounts.creator.wallet = ctx.accounts.creator_wallet.key();
    ctx.accounts.creator.created_at = Clock::get()?.unix_timestamp as u64;
    ctx.accounts.creator.fee_type = fee_type;
    ctx.accounts.creator.fee_amount = fee_amount;
    ctx.accounts.creator.fee_wallet = ctx.accounts.fee_wallet.key();

    Ok(())
}