pub mod errors;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use instructions::*;
pub use state::*;

declare_id!("8FRkaFSv9xwcrTE7noxzFkWQbVr7A4DUeUCb8CzdtrMS");

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
