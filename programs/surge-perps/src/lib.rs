pub mod errors;
pub mod state;
pub mod instructions;
pub mod events;

use anchor_lang::prelude::*;

pub use instructions::*;
pub use state::*;
pub use events::*;

declare_id!("GjMYNFqnZbAVoUoKxsYSaexT3AnXmHz1m8nMjH9Wxmdu");

#[program]
pub mod surge_perps {
    use super::*;

    pub fn open_position(
        ctx: Context<OpenPosition>,
        direction: state::Side,
        margin: u64,
    ) -> Result<()> {
        OpenPosition::open_position(ctx, direction, margin)
    }

    pub fn close_position(
        ctx: Context<ClosePosition>,
        check_token_balance: bool,
    ) -> Result<()> {
        ClosePosition::close_position(ctx, check_token_balance)
    }

    pub fn init_synthetic_mint(
        ctx: Context<InitSyntheticMint>,
    ) -> Result<()> {
        InitSyntheticMint::init_synthetic_mint(ctx)
    }

    pub fn set_vault(
        ctx: Context<SetVault>,
    ) -> Result<()> {
        SetVault::set_vault(ctx)
    }
} 
