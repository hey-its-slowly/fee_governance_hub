/// constants for seeds
pub const CONFIG_TAG:&[u8] = b"CONFIG_TAG";

/// constants for admin wallets
pub const ADMINS: [&str; 1] = [
  "F1tyGduCd9XLBSw2uAFycdNRUucyr2C7MiJ1Nifm2rZ8"
];

pub const MAX_FEE_WALLETS_LEN: usize = 3;
pub const MAX_FEE_INSTRUCTION_NAME_LEN: usize = 30;

pub const PERCENT_DENOMINATOR: u64 = 1000;

pub const GLOBAL_FEE_WALLETS: [&FeeWallet; 3] = [
  FeeWallet {
    address: Pubkey::from_str("ArpaDqpkJpKfxLP7WoFvYMbkj33C1PAHcy8tyrxFpgrc").unwrap(),
    fee_percent: 1000, // 100%
  },
  FeeWallet {
    address: Pubkey::from_str("11111111111111111111111111111111").unwrap(),
    fee_percent: 0,
  },
  FeeWallet {
    address: Pubkey::from_str("11111111111111111111111111111111").unwrap(),
    fee_percent: 0,
  },
];
