use anchor_lang::prelude::*;
use std::str::FromStr;

#[account]
pub struct Auction {
    pub creator: Pubkey,                // Creator of the auction
    pub nft_mint: Pubkey,              // NFT being auctioned
    pub accepted_mint: Pubkey,         // Token/NFT used for bidding
    pub tag: u64,                      // Tag (optional) default 0
    pub ended: bool,                   // Auction ended flag
    pub start_price: u64,              // Minimum starting bid
    pub current_bid: u64,              // Current highest bid
    pub current_winner: Option<Pubkey>, // Current winning bidder
    pub start_time: i64,               // Auction start timestamp
    pub end_time: i64,                 // Auction end timestamp
    pub destination: Pubkey,           // Where auction proceeds go
    pub burn_proceeds: bool,           // Whether to burn proceeds
    pub prize_type: u8,                 // Prize type for future use - 1: NFT, 2: CORE
    pub num_bids: u64,                 // Number of bids
    pub collection: Pubkey,    // Collection of the NFT
    pub tick_option: u8,               // Tick option
    pub tick_amount: u64,              // Tick amount
    pub bump: u8,                      // PDA bump
    pub reserved: [u128; 5],          // Reserved space for future use
}

#[account]
pub struct BidderInfo {
    pub bidder: Pubkey,               // Bidder's wallet
    pub bid_amount: u64,              // Amount bid
    pub bump: u8,                     // PDA bump
}

#[account]
#[derive(Default)]
pub struct Creator {
    pub bump: u8,
    pub wallet: Pubkey,
    pub fee_type: u8,
    pub fee_amount: u64,
    pub fee_wallet: Pubkey,

    pub created_at: u64,
    pub reserved: [u128; 1],
}

pub const SUPER_ADMIN: &str = "HyQpApazdwCD6DhSWJtymqvWz9c4nNiuemRG4Z1U5vbj";
pub const TICK_OPTION_PERCENTAGE: u8 = 1;
pub const TICK_OPTION_FLAT: u8 = 2;

pub const FEE_OPTION_NONE: u8 = 0;
pub const FEE_OPTION_PERCENTAGE: u8 = 1;
pub const FEE_OPTION_FLAT: u8 = 2;

pub fn is_super_admin(key: &Pubkey) -> bool {
    let key_from_str = Pubkey::from_str(SUPER_ADMIN);
    if key_from_str.is_ok() {
        return key_from_str.unwrap().eq(key);
    } else {
        return false;
    }
}

impl Auction {
    pub fn is_native_accepted_mint(&self) -> bool {
        let key_from_str = Pubkey::from_str("So11111111111111111111111111111111111111112");
        if key_from_str.is_ok() {
            return key_from_str.unwrap().eq(&self.accepted_mint);
        } else {
            return false;
        }
    }
}

impl Creator {
    pub fn is_creator_available(&self, authority: Pubkey) -> Result<bool> {
        if self.wallet == authority || is_super_admin(&authority) {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}