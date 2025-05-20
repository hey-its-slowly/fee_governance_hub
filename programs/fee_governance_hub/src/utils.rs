use anchor_lang::prelude::*;
use crate::constant::*;
use std::str::FromStr;

pub fn is_admin(key: &Pubkey)->bool {
    let mut authorities = vec![];
    for admin in ADMINS {
        authorities.push(Pubkey::from_str(admin).unwrap())
    }
    return authorities.contains(key)
}
