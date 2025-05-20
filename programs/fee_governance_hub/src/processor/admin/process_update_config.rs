use {
    crate::{constant::*, error::ContractError, state::*, utils::*},
    anchor_lang::prelude::*,
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct UpdateConfigIx {
    fee_instruction_index: u64,
    is_using_global_fee_wallets: bool,
    fee_wallets: [FeeWallet; MAX_FEE_WALLETS_LEN],
    fee_amount: u64,
    fee_instruction_name: String,
}

#[derive(Accounts)]
#[instruction(ix: UpdateConfigIx)]
pub struct UpdateConfigCtx<'info> {
    #[account(
      mut,
      constraint = is_admin(authority.key) @ ContractError::InvalidAuthority
    )]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [CONFIG_TAG, target_program.key().as_ref(), &ix.fee_instruction_index.to_le_bytes()],
        bump,
    )]
    pub config: Box<Account<'info, Config>>,

    /// CHECK: We read this key only
    pub target_program: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<UpdateConfigCtx>, ix: CreateConfigIx) -> Result<()> {
    let config = &mut ctx.accounts.config;

    config.is_using_global_fee_wallets = ix.is_using_global_fee_wallets;
    config.fee_wallets = ix.fee_wallets;
    config.fee_amount = ix.fee_amount;
    config.fee_instruction_name = ix.fee_instruction_name;
    
    Ok(())
}
