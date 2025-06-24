use anchor_lang::prelude::*;

declare_id!("GP4X2kmAFaKinvpZHeM3wkTJRbj6ARZ7bEfve1JfhpWw");

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
/// event
pub mod event;

use crate::processor::*;

#[program]
pub mod spl_fishing {
    use super::*;

    // super admin
    pub fn create_colleague(ctx: Context<CreateColleagueCtx>) -> Result<()> {
        process_create_colleague::handler(ctx)
    }

    pub fn remove_colleague(ctx: Context<RemoveColleagueCtx>) -> Result<()> {
        process_remove_colleague::handler(ctx)
    }

    pub fn send_payout(ctx: Context<SendPayoutCtx>, ix: SendPayoutIx) -> Result<()> {
        process_send_payout::handler(ctx, ix)
    }

    // admin
    pub fn create_game<'info>(ctx: Context<'_, '_, '_, 'info, CreateGameCtx<'info>>, ix: CreateGameIx) -> Result<()> {
        process_create_game::handler(ctx, ix)
    }

    pub fn edit_game(ctx: Context<EditGameCtx>, ix: EditGameIx) -> Result<()> {
        process_edit_game::handler(ctx, ix)
    }

    pub fn init_game_vault(ctx: Context<InitGameVaultCtx>) -> Result<()> {
        process_init_game_vault::handler(ctx)
    }

    pub fn withdraw_payment(ctx: Context<WithdrawPaymentCtx>) -> Result<()> {
        process_withdraw_payment::handler(ctx)
    }

    pub fn add_reward(ctx: Context<AddRewardCtx>, ix: AddRewardIx) -> Result<()> {
        process_add_reward::handler(ctx, ix)
    }

    pub fn edit_reward(ctx: Context<EditRewardCtx>, ix: EditRewardIx) -> Result<()> {
        process_edit_reward::handler(ctx, ix)
    }

    pub fn deposit_reward(ctx: Context<DepositRewardCtx>, ix: DepositRewardIx) -> Result<()> {
        process_deposit_reward::handler(ctx, ix)
    }

    pub fn withdraw_reward(ctx: Context<WithdrawRewardCtx>, ix: WithdrawRewardIx) -> Result<()> {
        process_withdraw_reward::handler(ctx, ix)
    }

    pub fn remove_reward(ctx: Context<RemoveRewardCtx>, ix: RemoveRewardIx) -> Result<()> {
        process_remove_reward::handler(ctx, ix)
    }

    // user
    pub fn flip<'info>(ctx: Context<'_, '_, '_, 'info, FlipCtx<'info>>) -> Result<()> {
        process_flip::handler(ctx)
    }
}
