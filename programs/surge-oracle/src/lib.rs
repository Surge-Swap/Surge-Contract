pub mod errors;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use instructions::*;
pub use state::*;

declare_id!("Dt3xxWhMg9RSvYyWwekqyU1jG7v7JKomMZ9seDPZU4L1");

#[program]
pub mod surge_oracle {
    use super::*;

    pub fn initialize_volatility_stats(ctx: Context<Initialize>) -> Result<()> {
        Initialize::initialize_volatility_stats(ctx)
    }

    pub fn update_volatility(ctx: Context<UpdateVolatility>) -> Result<()> {
        UpdateVolatility::update_volatility(ctx)
    }
}
