use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, Default)]
pub struct MarketBumps {
    pub market: u8,
}

#[account]
pub struct Market {
    pub epoch: u64,
    pub strike: u64,
    pub realized_variance: u64,
    pub var_long_mint: Pubkey,
    pub var_short_mint: Pubkey,
    pub usdc_vault: Pubkey,
    pub authority: Pubkey,
    pub volatility_stats: Pubkey,
    pub start_volatility: f64,
    pub bumps: MarketBumps,
    pub is_initialized: bool,
    pub is_expired: bool,
    pub total_deposits: u64,
}
