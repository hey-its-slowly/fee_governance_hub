use {
    anchor_lang::prelude::*
};

#[repr(C)]
#[derive(Clone, AnchorSerialize, AnchorDeserialize, PartialEq, Debug)]
pub struct FeeWallet {
    pub address: Pubkey,
    pub fee_percent: u64,
}

#[account]
#[derive(Default)]
pub struct Config {
    pub bump: u8,
    pub program: Pubkey,
    pub fee_instruction_index: u8,
    pub is_using_global_fee_wallets: bool,
    pub fee_amount: u64,
    pub fee_wallets: Vec<FeeWallet>,
    pub fee_instruction_name: String,

    pub created_at: u64,
    pub reserved: [u128; 2],
}
