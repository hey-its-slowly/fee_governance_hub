use anchor_lang::prelude::*;

#[event]
pub struct GameEvent {
    pub message: String,
    pub authority: Pubkey,
    pub game: Pubkey,
    pub game_id: u64,
    pub payment_token_mint: Pubkey,
    pub payment_token_decimals: u8,
    pub payment_token_unit_value: u64,
    pub payment_token_amount: u64,
    pub created_at: u64,
}

#[event]
pub struct RewardEvent {
    pub message: String,
    pub game: Pubkey,
    pub id: u8,
    pub mint: Pubkey,
    pub decimals: u8,
    pub unit_value: u64,
    pub rarity: u8,
    pub num_units: u64,
}
