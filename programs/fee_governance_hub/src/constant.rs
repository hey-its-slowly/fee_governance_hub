use anchor_lang::prelude::*;
use crate::state::FeeWallet;
use std::str::FromStr;

/// constants for seeds
pub const CONFIG_TAG:&[u8] = b"CONFIG_TAG";

/// constants for admin wallets
pub const ADMINS: [&str; 1] = [
  "F1tyGduCd9XLBSw2uAFycdNRUucyr2C7MiJ1Nifm2rZ8"
];

pub const MAX_FEE_WALLETS_LEN: usize = 3;
pub const MAX_FEE_INSTRUCTION_NAME_LEN: usize = 30;

pub const PERCENT_DENOMINATOR: u64 = 1000;

pub const GLOBAL_FEE_WALLETS: [&str; 3] = [
  "ArpaDqpkJpKfxLP7WoFvYMbkj33C1PAHcy8tyrxFpgrc",
  "11111111111111111111111111111111",
  "11111111111111111111111111111111"
];

pub const GLOBAL_FEE_WALLETS_FEE_PERCENT: [u64; 3] = [
  1000,
  0,
  0
];
