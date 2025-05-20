use {
    crate::{constant::*, error::ContractError, state::*, utils::*},
    anchor_lang::prelude::*,
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct TransferFeesIx {
    fee_instruction_index: u64,
}

#[derive(Accounts)]
#[instruction(ix: TransferFeesIx)]
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
    let config = ctx.accounts.config;

    let mut accumalated_percent = 0;
    for (index, account) in ctx.remaining_accounts.iter().enumerate() {
        let fee_wallet_info = if config.is_using_global_fee_wallets {
            config.fee_wallets[index]
        } else {
            GLOBAL_FEE_WALLETS[index]
        };
        
        if fee_wallet_info.address != account.key() {
            throw!(ContractError::InvalidFeeWallet);
        }

        let fee_amount = fee_wallet_info.fee_percent.checked_mul(config.fee_amount).unwrap().checked_div(PERCENT_DENOMINATOR).unwrap();

        anchor_lang::system_program::transfer(
            CpiContext::new(ctx.accounts.system_program.to_account_info(),
            anchor_lang::system_program::Transfer {
                from: ctx.accounts.authority.to_account_info(),
                to: account.to_account_info(),
            }
        ), fee_amount)?;

        accumalated_percent = accumalated_percent.checked_add(fee_wallet_info.fee_percent).unwrap();
    }

    require!(accumalated_percent == PERCENT_DENOMINATOR, ContractError::InvalidRemainingAccounts);

    Ok(())
}
