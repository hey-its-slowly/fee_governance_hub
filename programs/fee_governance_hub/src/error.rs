
use anchor_lang::prelude::*;

#[error_code]
pub enum ContractError {
    #[msg("Invalid Authority.")]
    InvalidAuthority,

    #[msg("Invalid Instruction.")]
    InvalidInstruction,

    #[msg("Invalid Fee Wallet.")]
    InvalidFeeWallet,

    #[msg("Invalid Remaining Accounts.")]
    InvalidRemainingAccounts,
}
