use anchor_lang::prelude::*;

#[event]
pub struct FlipEvent {
    pub message: String,
    pub game: Pubkey,
    pub game_flip_index: u64, // game play index
    pub player: Pubkey,
    pub reward_id: u8,
    pub winning_rarity: u8,
    pub reward_token_mint: Pubkey,
    pub reward_token_amount: u64,
    pub reward_token_decimals: u8,
}
