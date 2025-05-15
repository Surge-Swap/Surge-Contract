use anchor_lang::prelude::*;

#[account]
pub struct UserPosition {
    pub owner: Pubkey,               // User who owns this position
    pub entry_volatility: f64,       // Volatility at entry
    pub tokens_minted: u64,          // Number of tokens minted
    pub usdc_collateral: u64,        // USDC deposited as collateral
    pub mint_timestamp: i64,         // When position was created
    pub bump: u8,                    // PDA bump
}