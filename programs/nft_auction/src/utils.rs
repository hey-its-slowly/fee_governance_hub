use crate::account::{FEE_DENOMINATOR, FEE_OPTION_FLAT, FEE_OPTION_NONE, FEE_OPTION_PERCENTAGE};

/// Calculates the fee based on type and value.
pub fn calculate_fee(fee_type: u8, fee_amount: u64, amount: u64, decimals: u8) -> u64 {
    match fee_type {
        FEE_OPTION_NONE => 0,
        FEE_OPTION_PERCENTAGE => amount.saturating_mul(fee_amount) / 100,
        FEE_OPTION_FLAT => ((fee_amount as u128) * 10_u128.pow(decimals as u32) / (FEE_DENOMINATOR as u128)) as u64,
        _ => 0,
    }
}
