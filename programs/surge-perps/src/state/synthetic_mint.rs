use anchor_lang::prelude::*;

#[account]
pub struct SyntheticMint {
    pub bump: u8,
}

impl SyntheticMint {
    pub const LEN: usize = 8 + // discriminator
                           1;  // bump
}  