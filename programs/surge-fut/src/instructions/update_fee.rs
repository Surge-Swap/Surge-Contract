use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

use crate::{state::*, errors::ContractError};

#[derive(Accounts)]
#[instruction(new_fee_bps: u16)]
pub struct UpdateFee<'info> {
    #[account(
        constraint = authority.key() == token_config.authority @ ContractError::Unauthorized,
    )]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"token_config", token_mint.key().as_ref()],
        bump = token_config.bump,
    )]
    pub token_config: Account<'info, TokenConfig>,
    
    /// Volatility token mint
    #[account(
        constraint = token_mint.key() == token_config.token_mint @ ContractError::InvalidOracleData,
    )]
    pub token_mint: Account<'info, Mint>,
}

pub fn update_fee(ctx: Context<UpdateFee>, new_fee_bps: u16) -> Result<()> {
    // Validate fee percentage (maximum 10%)
    require!(new_fee_bps <= 10000, ContractError::InvalidFeePercentage);
    
    // Update fee
    ctx.accounts.token_config.fee_bps = new_fee_bps;
    
    msg!("Fee updated to: {}", new_fee_bps);
    
    Ok(())
}

