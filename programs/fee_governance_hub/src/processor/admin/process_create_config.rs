use {
    crate::{constant::*, error::ContractError, state::*, utils::*},
    anchor_lang::prelude::*,
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreateConfigIx {
    fee_instruction_index: u64,
    is_using_global_fee_wallets: bool,
    fee_wallets: [FeeWallet; MAX_FEE_WALLETS_LEN],
    fee_amount: u64,
    fee_instruction_name: String,
}

#[derive(Accounts)]
#[instruction(ix: CreateConfigIx)]
pub struct CreateConfigCtx<'info> {
    #[account(
      mut,
      constraint = is_admin(authority.key) @ ContractError::InvalidAuthority
    )]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = std::mem::size_of::<Config>() + 8 + MAX_FEE_WALLETS_LEN * std::mem::size_of::<FeeWallet>() + MAX_FEE_INSTRUCTION_NAME_LEN,
        seeds = [CONFIG_TAG, target_program.key().as_ref(), &ix.fee_instruction_index.to_le_bytes()],
        bump,
    )]
    pub config: Box<Account<'info, Config>>,

    /// CHECK: We read this key only
    pub target_program: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<CreateConfigCtx>, ix: CreateConfigIx) -> Result<()> {
    let config = &mut ctx.accounts.config;

    config.bump = ctx.bumps.config;
    config.program = ctx.accounts.target_program.key();
    config.fee_instruction_index = ix.fee_instruction_index as u8;
    config.is_using_global_fee_wallets = ix.is_using_global_fee_wallets;
    config.fee_wallets = ix.fee_wallets.to_vec();
    config.fee_amount = ix.fee_amount;
    config.fee_instruction_name = ix.fee_instruction_name;
    
    config.created_at = Clock::get()?.unix_timestamp as u64;

    Ok(())
}
