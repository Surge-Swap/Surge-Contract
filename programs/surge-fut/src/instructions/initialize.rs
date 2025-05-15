use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use anchor_spl::associated_token::AssociatedToken;

use crate::{state::*, errors::ContractError};

#[derive(Accounts)]
#[instruction(token_name: String, token_symbol: String, fee_bps: u16)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        mint::decimals = 6,
        mint::authority = token_config,
    )]
    pub token_mint: Account<'info, Mint>,
    
    /// USDC mint
    #[account(
        constraint = usdc_mint.decimals == 6 @ ContractError::InvalidOracleData,
    )]
    pub usdc_mint: Account<'info, Mint>,
    
    /// Fee destination account - must be an existing USDC token account owned by authority
    #[account(
        mut,
        constraint = fee_destination.mint == usdc_mint.key() @ ContractError::InvalidOracleData,
        constraint = fee_destination.owner == authority.key() @ ContractError::Unauthorized,
    )]
    pub fee_destination: Account<'info, TokenAccount>,
    
    /// Collateral pool account
    #[account(
        init_if_needed,
        payer = authority,
        seeds = [b"collateral_pool", token_mint.key().as_ref()],
        bump,
        token::mint = usdc_mint,
        token::authority = token_config,
    )]
    pub collateral_pool: Account<'info, TokenAccount>,
    
    /// Oracle account with volatility data
    /// CHECK: This account is validated in the handler
    pub oracle: AccountInfo<'info>,
    
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 32 + 32 + 32 + 32 + 32 + 2 + 32 + 8 + 8 + 1 + 1 + 200, // Extra space for name/symbol and collateral_pool_bump
        seeds = [b"token_config", token_mint.key().as_ref()],
        bump
    )]
    pub token_config: Account<'info, TokenConfig>,
    
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn initialize(
    ctx: Context<Initialize>,
    token_name: String,
    token_symbol: String,
    fee_bps: u16,
) -> Result<()> {
    // Validate fee percentage
    require!(fee_bps <= 10000, ContractError::InvalidFeePercentage);
    
    // Try to get the current volatility from oracle
    let volatility = VolatilityStats::load_from_account_info(&ctx.accounts.oracle)?;
    msg!("Current volatility: {}", volatility);
    
    // Validate volatility data
    require!(volatility > 0.0, ContractError::InvalidOracleData);
    
    // Initialize token config state
    let token_config = &mut ctx.accounts.token_config;
    token_config.authority = ctx.accounts.authority.key();
    token_config.token_mint = ctx.accounts.token_mint.key();
    token_config.usdc_mint = ctx.accounts.usdc_mint.key();
    token_config.fee_destination = ctx.accounts.fee_destination.key();
    token_config.collateral_pool = ctx.accounts.collateral_pool.key();
    token_config.collateral_pool_bump = ctx.bumps.collateral_pool;
    token_config.token_name = token_name;
    token_config.token_symbol = token_symbol;
    token_config.fee_bps = fee_bps;
    token_config.oracle = ctx.accounts.oracle.key();
    token_config.total_tokens_outstanding = 0;
    token_config.usdc_per_vol_point = 100_000; // 0.1 USDC per 0.001 volatility point (adjustable)
    token_config.bump = ctx.bumps.token_config;
    
    msg!("Token config initialized successfully: {}", token_config.token_name);
    
    Ok(())
}