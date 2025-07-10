use anchor_lang::prelude::*;

declare_id!("Ad1y5v7dsQyiJF1cztiy9nUS2skKGC3apf8ngcacnQS8");

/// constant
pub mod constant;
/// error
pub mod error;
/// utils
pub mod utils;
/// processor
pub mod processor;
/// state
pub mod state;

use crate::processor::*;

#[program]
pub mod nft_auction {
    use super::*;

    // admin
    pub fn add_creator(ctx: Context<AddCreator>, fee_type: u8, fee_amount: u64) -> Result<()> {
        process_add_creator::handler(ctx, fee_type, fee_amount)
    }

    pub fn edit_creator(ctx: Context<EditCreator>, fee_type: u8, fee_amount: u64) -> Result<()> {
        process_edit_creator::handler(ctx, fee_type, fee_amount)
    }

    pub fn remove_creator(ctx: Context<RemoveCreator>) -> Result<()> {
        process_remove_creator::handler(ctx)
    }

    // creator
    pub fn create_auction(
        ctx: Context<CreateAuction>,
        start_price: u64,
        start_time: i64,
        end_time: i64,
        destination: Option<Pubkey>,
        burn_proceeds: bool,
        tag: u64,
        tick_option: u8,
        tick_amount: u64,
    ) -> Result<()> {
        process_create_auction::handler(
            ctx,
            start_price,
            start_time,
            end_time,
            destination,
            burn_proceeds,
            tag,
            tick_option,
            tick_amount,
        )
    }

    pub fn create_auction_v2(
        ctx: Context<CreateAuctionV2>,
        start_price: u64,
        start_time: i64,
        end_time: i64,
        destination: Option<Pubkey>,
        burn_proceeds: bool,
        tag: u64,
        tick_option: u8,
        tick_amount: u64,
    ) -> Result<()> {
        process_create_auction_v2::handler(
            ctx,
            start_price,
            start_time,
            end_time,
            destination,
            burn_proceeds,
            tag,
            tick_option,
            tick_amount,
        )
    }

    pub fn create_auction_v3<'info>(
        ctx: Context<'_, '_, '_, 'info, CreateAuctionV3<'info>>,
        start_price: u64,
        start_time: i64,
        end_time: i64,
        destination: Option<Pubkey>,
        burn_proceeds: bool,
        tag: u64,
        tick_option: u8,
        tick_amount: u64,
        root: [u8; 32],
        data_hash: [u8; 32],
        creator_hash: [u8; 32],
        nonce: u64,
        index: u32,
    ) -> Result<()> {
        process_create_auction_v3::handler(ctx, start_price, start_time, end_time, destination, burn_proceeds, tag, tick_option, tick_amount, root, data_hash, creator_hash, nonce, index)
    }

    pub fn init_auction_vault(ctx: Context<InitAuctionVault>) -> Result<()> {
        process_init_auction_vault::handler(ctx)
    }

    pub fn init_auction_vault_2022(ctx: Context<InitAuctionVault2022>) -> Result<()> {
        process_init_auction_vault_2022::handler(ctx)
    }

    pub fn cancel_auction(ctx: Context<CancelAuction>) -> Result<()> {
        process_cancel_auction::handler(ctx)
    }

    pub fn cancel_auction_2022(ctx: Context<CancelAuction2022>) -> Result<()> {
        process_cancel_auction_2022::handler(ctx)
    }

    pub fn cancel_auction_v2(ctx: Context<CancelAuctionV2>) -> Result<()> {
        process_cancel_auction_v2::handler(ctx)
    }

    pub fn cancel_auction_v2_2022(ctx: Context<CancelAuctionV22022>) -> Result<()> {
        process_cancel_auction_v2_2022::handler(ctx)
    }

    pub fn cancel_auction_v3<'info>(
        ctx: Context<'_, '_, '_, 'info, CancelAuctionV3<'info>>,
        root: [u8; 32],
        data_hash: [u8; 32],
        creator_hash: [u8; 32],
        nonce: u64,
        index: u32,
    ) -> Result<()> {
        process_cancel_auction_v3::handler(ctx, root, data_hash, creator_hash, nonce, index)
    }

    pub fn cancel_auction_v3_2022<'info>(
        ctx: Context<'_, '_, '_, 'info, CancelAuctionV32022<'info>>,
        root: [u8; 32],
        data_hash: [u8; 32],
        creator_hash: [u8; 32],
        nonce: u64,
        index: u32,
    ) -> Result<()> {
        process_cancel_auction_v3_2022::handler(ctx, root, data_hash, creator_hash, nonce, index)
    }

    // user
    pub fn place_bid(ctx: Context<PlaceBid>, bid_amount: u64) -> Result<()> {
        process_place_bid::handler(ctx, bid_amount)
    }

    pub fn place_bid_2022(ctx: Context<PlaceBid2022>, bid_amount: u64) -> Result<()> {
        process_place_bid_2022::handler(ctx, bid_amount)
    }

    pub fn claim_nft(ctx: Context<ClaimNft>) -> Result<()> {
        process_claim_nft::handler(ctx)
    }

    pub fn claim_nft_2022(ctx: Context<ClaimNft2022>) -> Result<()> {
        process_claim_nft_2022::handler(ctx)
    }

    pub fn claim_nft_v2(ctx: Context<ClaimNftV2>) -> Result<()> {
        process_claim_nft_v2::handler(ctx)
    }

    pub fn claim_nft_v2_2022(ctx: Context<ClaimNftV22022>) -> Result<()> {
        process_claim_nft_v2_2022::handler(ctx)
    }

    pub fn claim_nft_v3<'info>(
        ctx: Context<'_, '_, '_, 'info, ClaimNftV3<'info>>,
        root: [u8; 32],
        data_hash: [u8; 32],
        creator_hash: [u8; 32],
        nonce: u64,
        index: u32,
    ) -> Result<()> {
        process_claim_nft_v3::handler(ctx, root, data_hash, creator_hash, nonce, index)
    }

    pub fn claim_nft_v3_2022<'info>(
        ctx: Context<'_, '_, '_, 'info, ClaimNftV32022<'info>>,
        root: [u8; 32],
        data_hash: [u8; 32],
        creator_hash: [u8; 32],
        nonce: u64,
        index: u32,
    ) -> Result<()> {
        process_claim_nft_v3_2022::handler(ctx, root, data_hash, creator_hash, nonce, index)
    }
}
