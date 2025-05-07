pub mod errors;
pub mod state;
pub mod instructions;
pub mod events;

use anchor_lang::prelude::*;

pub use instructions::*;
pub use state::*;
pub use events::*;

declare_id!("3gZPZnYVM8BT25H9VJUX48jqdP56J5yLoAwzz93i7uRt");

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
    ) -> Result<()> {
        ClosePosition::close_position(ctx)
    }
} 
