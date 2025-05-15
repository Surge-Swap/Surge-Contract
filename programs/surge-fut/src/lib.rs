use anchor_lang::prelude::*;

pub mod errors;
pub mod state;
pub mod instructions;

use instructions::*;

declare_id!("CarydvHuPVR4TZbnPQjnEbrNWXFohefCYHEoWsZMPDvZ");

#[program]
pub mod surge_fut {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>, 
        token_name: String, 
        token_symbol: String, 
        fee_bps: u16
    ) -> Result<()> {
       instructions::initialize::initialize(ctx, token_name, token_symbol, fee_bps)
    }

    pub fn mint_tokens(
        ctx: Context<MintTokens>,
        amount: u64,
    ) -> Result<()> {
        instructions::mint_tokens::mint_tokens(ctx, amount)
    }

    pub fn redeem_tokens(
        ctx: Context<RedeemTokens>,
        amount: u64,
    ) -> Result<()> {
        instructions::redeem_tokens::redeem_tokens(ctx, amount)
    }

    pub fn update_fee(
        ctx: Context<UpdateFee>,
        new_fee_bps: u16,
    ) -> Result<()> {
        instructions::update_fee::update_fee(ctx, new_fee_bps)
    }
}

