use anchor_lang::prelude::*;


#[error_code]
pub enum AuctionCode {
    #[msg("Invalid authority")]
    InvalidAuthority,
    #[msg("Auction start time must be in the future")]
    InvalidStartTime,
    #[msg("Auction end time must be after start time")]
    InvalidEndTime,
    #[msg("Auction has not started yet")]
    AuctionNotStarted,
    #[msg("Auction has ended")]
    AuctionEnded,
    #[msg("Bid amount is too low")]
    BidTooLow,
    #[msg("Auction has not ended yet")]
    AuctionNotEnded,
    #[msg("NFT has already been claimed")]
    AlreadyClaimed,
    #[msg("Only the auction winner or creator (if no bids) can claim")]
    UnauthorizedClaimer,
    #[msg("Invalid destination account")]
    InvalidDestinationAccount,
    #[msg("Invalid destination mint")]
    InvalidDestinationMint,
    #[msg("Cannot cancel auction with active bids")]
    AuctionHasBids,
    #[msg("Only the auction creator can cancel")]
    UnauthorizedCanceller,
    #[msg("Invalid creator")]
    InvalidCreator,
    #[msg("Invalid previous bidder")]
    InvalidPreviousBidder,
    #[msg("Invalid mint")]
    InvalidMint,
    #[msg("Invalid tick option")]
    InvalidTickOption,
}
