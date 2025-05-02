use anchor_lang::prelude::*;

#[account]
pub struct VolatilityStats {
    pub authority: Pubkey,
    pub last_price: u64,            // Fixed-point price (1e6)
    pub mean: f64,                  // Mean of log-returns
    pub m2: f64,                    // Running Σ(r - mean)^2
    pub count: u64,                 // Number of returns seen
    pub annualized_volatility: f64, // Annualized σ estimate
}
