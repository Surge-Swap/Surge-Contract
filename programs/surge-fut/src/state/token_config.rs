use anchor_lang::prelude::*;

#[account]
pub struct TokenConfig {
    pub authority: Pubkey,           // Admin who can update fees
    pub token_mint: Pubkey,          // The mint for our volatility token
    pub usdc_mint: Pubkey,           // USDC mint address
    pub fee_destination: Pubkey,     // Where fees go
    pub collateral_pool: Pubkey,     // Where collateral is stored
    pub token_name: String,          // Name of the token
    pub token_symbol: String,        // Symbol of the token
    pub fee_bps: u16,                // Fee in basis points (1/100 of 1%)
    pub oracle: Pubkey,              // Volatility oracle address
    pub total_tokens_outstanding: u64, // Total number of tokens minted
    pub usdc_per_vol_point: u64,     // How much USDC per 1% point of volatility
    pub collateral_pool_bump: u8,    // Bump for the collateral pool PDA
    pub bump: u8,                    // PDA bump
}