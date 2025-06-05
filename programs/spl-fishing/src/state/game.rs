use anchor_lang::prelude::*;

#[derive(Clone, AnchorSerialize, AnchorDeserialize, PartialEq, Debug)]
pub struct RewardTokenInfo {
    pub id: u8,
    pub mint: Pubkey,
    pub unit_value: u64,
    pub decimals: u8,
    pub rarity: u8, // 1: Common, 2: Uncommon, 3: Rare, 4: Epic
    pub num_units: u64,
    pub num_winners: u64,
}

#[account]
#[derive(Default)]
pub struct Game {
    pub bump: u8,
    pub authority: Pubkey, // game creator's wallet
    pub game_id: u64,

    pub payment_token_mint: Pubkey, // Token mint that is being deposited
    pub payment_token_decimals: u8, // Decimals of payment_token_mint
    pub payment_token_unit_value: u64, // one-time cost of payment_token_mint
    pub payment_token_amount: u64, // Amount of payment_token_mint in game

    pub num_flips: u64,

    pub reward_tokens: Vec<RewardTokenInfo>, // Maximum 10
    pub reveal_pending_players: Vec<Pubkey>, // Maximum 10 is enough - the number of players that can play at the same time. release when claim rewards.

    pub created_at: u64,
    
    pub reserved: [u128; 5],
}

impl Game {
    pub fn has_rarity(&self, rarity: u8) -> bool {
        self.reward_tokens.iter().any(|reward_token_info| reward_token_info.rarity == rarity)
    }

    pub fn get_reward_token_index(&self, reward_id: u8) -> usize {
        self.reward_tokens.iter()
            .position(|reward_token_info| reward_token_info.id == reward_id)
            .unwrap_or(usize::MAX)
    }
    
    pub fn is_flip_available(&self, player: Pubkey) -> bool { // check if the player can flip - reveal pending players len is less than all reward tokens' num_units
        self.reward_tokens.iter().all(|reward_token_info| reward_token_info.num_units > self.reveal_pending_players.len() as u64) && !self.reveal_pending_players.contains(&player)
    }
}