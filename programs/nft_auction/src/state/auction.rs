use anchor_lang::prelude::*;
use std::str::FromStr;

#[account]
#[derive(Default)]
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
    pub prize_type: u8,                 // Prize type for future use - 1: NFT, 2: CORE, 3: CNFT
    pub num_bids: u64,                 // Number of bids
    pub collection: Pubkey,    // Collection of the NFT
    pub tick_option: u8,               // Tick option
    pub tick_amount: u64,              // Tick amount
    pub bump: u8,                      // PDA bump
    pub reserved: [u128; 5],          // Reserved space for future use
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