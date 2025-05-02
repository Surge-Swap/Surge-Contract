pub mod errors;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use instructions::*;
pub use state::*;

declare_id!("5HEivNreiuWazK2wtLQTxsaT31rEsVcSJCpMiTSzMy77");

#[program]
pub mod surge_variance {
    use super::*;

    pub fn initialize_market(
        ctx: Context<InitializeMarket>,
        epoch: u64,
        strike: u64,
        bumps: MarketBumps,
    ) -> Result<()> {
        InitializeMarket::initialize_market(ctx, epoch, strike, bumps)
    }

    pub fn mint_tokens(ctx: Context<MintTokens>, amount: u64, is_long: bool) -> Result<()> {
        MintTokens::mint_tokens(ctx, amount, is_long)
    }

    pub fn redeem(ctx: Context<Redeem>) -> Result<()> {
        Redeem::redeem(ctx)
    }
}
