pub mod errors;
pub mod instructions;
pub mod state;
pub mod events;

use anchor_lang::prelude::*;

pub use instructions::*;
pub use state::*;
pub use events::*;

declare_id!("4aL6kUNn43DEwEdUvcjrDrofZwJNPYcfPZqoTZfg2BSk");

#[program]
pub mod surge_variance {
    use super::*;

    pub fn initialize_market(
        ctx: Context<InitializeMarket>,
        epoch: u64,
        strike: f64,
        timestamp: i64,
        bumps: MarketBumps,
    ) -> Result<()> {
        InitializeMarket::initialize_market(ctx, epoch, strike, timestamp, bumps)
    }

    pub fn mint_tokens(ctx: Context<MintTokens>, amount: u64, is_long: bool, epoch: u64, timestamp: i64, bumps: MarketBumps) -> Result<()> {
        MintTokens::mint_tokens(ctx, amount, is_long, epoch, timestamp, bumps)
    }

    pub fn redeem(ctx: Context<Redeem>, epoch: u64, timestamp: i64, bumps: MarketBumps) -> Result<()> {
        Redeem::redeem(ctx, epoch, timestamp, bumps)
    }
}
