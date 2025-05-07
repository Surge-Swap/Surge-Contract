use anchor_lang::prelude::*;
use crate::state::Side;

#[event]
pub struct PositionOpened {
    pub owner: Pubkey,
    pub position: Pubkey,
    pub direction: Side,
    pub entry_vol: f64,
    pub size: u64,
    pub margin: u64,
    pub timestamp: i64,
}

#[event]
pub struct PositionClosed {
    pub owner: Pubkey,
    pub position: Pubkey,
    pub direction: Side,
    pub entry_vol: f64,
    pub exit_vol: f64,
    pub size: u64,
    pub margin: u64,
    pub pnl: i64,
    pub payout: u64,
    pub timestamp: i64,
} 