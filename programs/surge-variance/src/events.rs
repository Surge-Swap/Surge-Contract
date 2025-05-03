use anchor_lang::prelude::*;

#[event]
pub struct MarketInitialized {
    pub market: Pubkey,
    pub authority: Pubkey,
    pub usdc_vault: Pubkey,
    pub var_long_mint: Pubkey,
    pub var_short_mint: Pubkey,
    pub epoch: u64,
    pub strike: f64,
    pub timestamp: i64,
    pub start_volatility: f64,
}

#[event]
pub struct TokensMinted {
    pub market: Pubkey,
    pub user: Pubkey,
    pub amount: u64,
    pub is_long: bool,
    pub total_deposits: u64,
}

#[event]
pub struct MarketRedeemed {
    pub market: Pubkey,
    pub user: Pubkey,
    pub realized_variance: f64,
    pub strike: f64,
    pub long_payout: u64,
    pub short_payout: u64,
    pub total_deposits: u64,
}
