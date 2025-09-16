use anchor_lang::prelude::*;
use crate::utils::is_super_admin;

#[account]
#[derive(Default)]
pub struct Creator {
    pub bump: u8,
    pub wallet: Pubkey,
    pub fee_type: u8,
    pub fee_amount: u64,
    pub fee_wallet: Pubkey,
    pub backend_authority: Pubkey,  // Optional backend authority for bid validation

    pub created_at: u64,
    pub reserved: [u128; 1],
}

impl Creator {
    pub fn is_creator_available(&self, authority: Pubkey) -> Result<bool> {
        if self.wallet == authority || is_super_admin(&authority) {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn requires_backend_authority(&self) -> bool {
        // If backend_authority is set to system program, no backend authority is required
        self.backend_authority != anchor_lang::system_program::ID
    }

    pub fn get_required_backend_authority(&self) -> Option<Pubkey> {
        if self.requires_backend_authority() {
            Some(self.backend_authority)
        } else {
            None
        }
    }
}