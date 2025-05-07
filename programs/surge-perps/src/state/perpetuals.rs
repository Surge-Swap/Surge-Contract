use anchor_lang::prelude::*;

#[account]
pub struct Position {
    pub owner: Pubkey,
    pub direction: Side,
    pub entry_vol: f64,
    pub size: u64,
    pub margin: u64,
    pub bump: u8,
    pub created_at: i64,
    pub is_active: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Copy)]
pub enum Side {
    Long,
    Short,
}

impl Position {
    pub const LEN: usize = 8 + 32 + 1 + 8 + 8 + 8 + 1 + 8 + 1;
}