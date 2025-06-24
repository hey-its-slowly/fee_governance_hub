use anchor_lang::prelude::*;

declare_id!("F6kQqFTchcczKgSJJLvVqB6jHrq67gchMW7Af7hjHycQ");

/// constant
pub mod constant;
/// error
pub mod error;
/// processor
pub mod processor;
/// states
pub mod state;
/// utils
pub mod utils;

use crate::processor::*;

#[program]
pub mod fee_governance_hub {
    use super::*;

    // admin
    pub fn create_config(ctx: Context<CreateConfigCtx>, ix: CreateConfigIx) -> Result<()> {
        process_create_config::handler(ctx, ix)
    }

    pub fn update_config(ctx: Context<UpdateConfigCtx>, ix: UpdateConfigIx) -> Result<()> {
        process_update_config::handler(ctx, ix)
    }
    
    // cpi
    pub fn transfer_fees<'info>(ctx: Context<'_, '_, '_, 'info, TransferFeesCtx<'info>>, ix: TransferFeesIx) -> Result<()> {
        process_transfer_fees::handler(ctx, ix)
    }
}
