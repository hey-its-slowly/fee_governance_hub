use {
    crate::{error::ContractError, state::*, utils::*},
    anchor_lang::prelude::*,
};

#[derive(Accounts)]
pub struct EditCreator<'info> {
    #[account(mut,
        constraint = is_super_admin(authority.key) @ ContractError::InvalidAuthority
    )]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"creator", creator_wallet.key().as_ref()],
        bump,
    )]
    pub creator: Box<Account<'info, Creator>>,

    /// CHECK: Not dangerous because only admin can send tx
    pub creator_wallet: UncheckedAccount<'info>,

    /// CHECK: Not dangerous because only admin can send tx
    pub fee_wallet: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<EditCreator>, fee_type: u8, fee_amount: u64, backend_authority: Option<Pubkey>) -> Result<()> {
  let creator = &mut ctx.accounts.creator;
  
  // Update creator fields
  creator.fee_type = fee_type;
  creator.fee_amount = fee_amount;
  creator.fee_wallet = ctx.accounts.fee_wallet.key();
  // Update backend_authority if provided, otherwise keep existing value
  if let Some(auth) = backend_authority {
    creator.backend_authority = auth;
  }

  msg!("Updated creator: {} with fee_type: {}, fee_amount: {}, fee_wallet: {}, backend_authority: {}", 
       ctx.accounts.creator_wallet.key(), fee_type, fee_amount, ctx.accounts.fee_wallet.key(), creator.backend_authority);

  Ok(())
}