use anchor_lang::prelude::*;
use crate::{
    error::ContractError,
    constant::*,
};
use std::str::FromStr;

pub fn is_super_admin(key: &Pubkey) -> bool {
    let key_from_str = Pubkey::from_str(SUPER_ADMIN);
    if key_from_str.is_ok() {
        return key_from_str.unwrap().eq(key);
    } else {
        return false;
    }
}

pub trait SafeCalc<T> {
    fn safe_add(&self, num: T) -> Result<T>;
    fn safe_sub(&self, num: T) -> Result<T>;
    fn safe_mul(&self, num: T) -> Result<T>;
    fn safe_div(&self, num: T) -> Result<T>;
    fn safe_pow(&self, num: u32) -> Result<T>;
}
impl SafeCalc<u64> for u64 {
    fn safe_add(&self, num: u64) -> Result<u64> {
        let result = self.checked_add(num);
        if result.is_none() {
            return Err(error!(ContractError::MathOverflow));
        }
        Ok(result.unwrap())
    }
    fn safe_sub(&self, num: u64) -> Result<u64> {
        let result = self.checked_sub(num);
        if result.is_none() {
            return Err(error!(ContractError::MathOverflow));
        }
        Ok(result.unwrap())
    }
    fn safe_mul(&self, num: u64) -> Result<u64> {
        let result = self.checked_mul(num);
        if result.is_none() {
            return Err(error!(ContractError::MathOverflow));
        }
        Ok(result.unwrap())
    }
    fn safe_div(&self, num: u64) -> Result<u64> {
        let result = self.checked_div(num);
        if result.is_none() {
            return Err(error!(ContractError::MathOverflow));
        }
        Ok(result.unwrap())
    }
    fn safe_pow(&self, num: u32) -> Result<u64> {
        let result = self.checked_pow(num);
        if result.is_none() {
            return Err(error!(ContractError::MathOverflow));
        }
        Ok(result.unwrap())
    }
}