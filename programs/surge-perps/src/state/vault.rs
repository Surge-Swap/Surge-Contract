use anchor_lang::prelude::*;

#[account]
pub struct VaultConfig {
    pub custom_vault: Pubkey,
    pub bump: u8,
}

impl VaultConfig {
    pub const LEN: usize = 8 + // discriminator
                           32 + // custom_vault pubkey
                           1;   // bump
} 