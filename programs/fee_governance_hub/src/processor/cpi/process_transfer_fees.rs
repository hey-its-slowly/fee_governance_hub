use {
    crate::{constant::*, error::ContractError, state::*, utils::*},
    anchor_lang::prelude::*,
    std::str::FromStr,
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct TransferFeesIx {
    pub fee_instruction_index: u64,
}

#[derive(Accounts)]
#[instruction(ix: TransferFeesIx)]
pub struct TransferFeesCtx<'info> {
    #[account(
      mut,
      constraint = is_admin(authority.key) @ ContractError::InvalidAuthority
    )]
    pub authority: Signer<'info>,

    #[account(
        seeds = [CONFIG_TAG, target_program.key().as_ref(), &ix.fee_instruction_index.to_le_bytes()],
        bump,
    )]
    pub config: Box<Account<'info, Config>>,

    /// CHECK: We read this key only
    pub target_program: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, TransferFeesCtx<'info>>, _: TransferFeesIx) -> Result<()> {
    let config = &ctx.accounts.config;
    
    let mut fee_wallets = Vec::new();
    if config.is_using_global_fee_wallets {
        for i in 0..GLOBAL_FEE_WALLETS.len() {
            let fee_wallet = FeeWallet {
                address: Pubkey::from_str(GLOBAL_FEE_WALLETS[i]).unwrap(),
                fee_percent: GLOBAL_FEE_WALLETS_FEE_PERCENT[i],
            };
            fee_wallets.push(fee_wallet);
        }
    } else {
        for i in 0..config.fee_wallets.len() {
            let fee_wallet = FeeWallet {
                address: config.fee_wallets[i].address,
                fee_percent: config.fee_wallets[i].fee_percent,
            };
            fee_wallets.push(fee_wallet);
        }
    }

    let mut accumalated_percent: u64 = 0;
    for (index, account) in ctx.remaining_accounts.iter().enumerate() {
        let fee_wallet_info = fee_wallets[index].clone();
        
        if fee_wallet_info.address != account.key() {
            return Err(ContractError::InvalidFeeWallet.into());
        }

        let fee_amount = fee_wallet_info.fee_percent.checked_mul(ctx.accounts.config.fee_amount).unwrap().checked_div(PERCENT_DENOMINATOR).unwrap();

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
