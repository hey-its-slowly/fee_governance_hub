use crate::account::{FEE_OPTION_NONE, FEE_OPTION_PERCENTAGE, FEE_OPTION_FLAT};

/// Calculates the fee based on type and value.
pub fn calculate_fee(fee_type: u8, fee_amount: u64, amount: u64) -> u64 {
    match fee_type {
        FEE_OPTION_NONE => 0,
        FEE_OPTION_PERCENTAGE => amount.saturating_mul(fee_amount) / 100,
        FEE_OPTION_FLAT => fee_amount,
        _ => 0,
    }
}
