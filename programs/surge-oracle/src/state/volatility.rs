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

impl VolatilityStats {
    pub const SIZE: usize = 8 +  // discriminator
        32 +  // authority
        8 +   // last_price
        8 +   // mean
        8 +   // m2
        8 +   // count
        8; // annualized_volatility

    pub fn update_volatility(
        &mut self,
        updated_last_price: Option<u64>,
        updated_mean: Option<f64>,
        updated_m2: Option<f64>,
        updated_count: Option<u64>,
        updated_annualized_volatility: Option<f64>,
    ) {
        if let Some(val) = updated_last_price {
            self.last_price = val;
        }
        if let Some(val) = updated_mean {
            self.mean = val;
        }
        if let Some(val) = updated_m2 {
            self.m2 = val;
        }
        if let Some(val) = updated_count {
            self.count = val;
        }
        if let Some(val) = updated_annualized_volatility {
            self.annualized_volatility = val;
        }
    }
}
