use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct Colleague { // Can create game without origination fee.
    pub bump: u8,
    pub wallet: Pubkey, // wallet of the colleague
    pub num_games: u64, // Number of games created by the colleague

    pub created_at: u64,
    
    pub reserved: [u128; 2],
}
