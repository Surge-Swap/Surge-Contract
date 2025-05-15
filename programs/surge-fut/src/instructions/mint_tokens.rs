use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use anchor_spl::associated_token::AssociatedToken;

use crate::{state::*, errors::ContractError};

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct MintTokens<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    /// USDC token account of the user
    #[account(
        mut,
        constraint = user_usdc_account.owner == user.key() @ ContractError::Unauthorized,
        constraint = user_usdc_account.mint == token_config.usdc_mint @ ContractError::InvalidOracleData,
    )]
    pub user_usdc_account: Account<'info, TokenAccount>,
    
    /// Fee destination USDC account
    #[account(
        mut,
        constraint = fee_destination.key() == token_config.fee_destination @ ContractError::Unauthorized,
    )]
    pub fee_destination: Account<'info, TokenAccount>,
    
    /// Collateral pool USDC account
    #[account(
        mut,
        seeds = [b"collateral_pool", token_mint.key().as_ref()],
        bump = token_config.collateral_pool_bump,
        constraint = collateral_pool.key() == token_config.collateral_pool @ ContractError::Unauthorized,
    )]
    pub collateral_pool: Account<'info, TokenAccount>,
    
    /// Volatility token account of the user
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = token_mint,
        associated_token::authority = user,
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    
    /// Volatility token mint
    #[account(
        mut,
        constraint = token_mint.key() == token_config.token_mint @ ContractError::InvalidOracleData,
    )]
    pub token_mint: Account<'info, Mint>,
    
    /// Token Config
    #[account(
        mut,
        seeds = [b"token_config", token_mint.key().as_ref()],
        bump = token_config.bump,
    )]
    pub token_config: Account<'info, TokenConfig>,
    
    /// User Position Account
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + 32 + 8 + 8 + 8 + 8 + 1,
        seeds = [b"user_position", user.key().as_ref(), token_mint.key().as_ref()],
        bump,
    )]
    pub user_position: Account<'info, UserPosition>,
    
    /// Oracle account with volatility data
    /// CHECK: Account is validated through VolatilityStats::load_from_account_info
    pub oracle: AccountInfo<'info>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn mint_tokens(ctx: Context<MintTokens>, amount: u64) -> Result<()> {
    // Validate amount
    require!(amount > 0, ContractError::InvalidAmount);
    
    // Get current volatility from oracle
    let current_volatility = VolatilityStats::load_from_account_info(&ctx.accounts.oracle)?;
    msg!("Current volatility: {}", current_volatility);
    
    // Calculate USDC required based on token amount and current volatility
    let usdc_per_vol = ctx.accounts.token_config.usdc_per_vol_point;
    
    // Convert f64 volatility to u64 representation (multiplied by 1000 to preserve precision)
    let vol_points = (current_volatility * 1000.0) as u64;
    
    // Calculate required USDC amount: amount * volatility * usdc_per_vol_point
    let usdc_required = amount
        .checked_mul(vol_points)
        .ok_or(ContractError::MathOverflow)?
        .checked_mul(usdc_per_vol)
        .ok_or(ContractError::MathOverflow)?
        .checked_div(1000) // Adjust for the volatility scaling
        .ok_or(ContractError::MathOverflow)?;
        
    msg!("USDC required: {}", usdc_required);
    
    // Calculate fee
    let fee_amount = usdc_required
        .checked_mul(ctx.accounts.token_config.fee_bps as u64)
        .ok_or(ContractError::MathOverflow)?
        .checked_div(10000)
        .ok_or(ContractError::MathOverflow)?;
    
    msg!("Fee amount: {}", fee_amount);
    
    // Total amount user needs to pay
    let total_payment = usdc_required
        .checked_add(fee_amount)
        .ok_or(ContractError::MathOverflow)?;
    
    // Check if user has enough USDC
    require!(
        ctx.accounts.user_usdc_account.amount >= total_payment,
        ContractError::InsufficientBalance
    );
    
    // Transfer fee to fee destination
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_usdc_account.to_account_info(),
                to: ctx.accounts.fee_destination.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        fee_amount,
    )?;
    
    // Transfer collateral to the collateral pool
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_usdc_account.to_account_info(),
                to: ctx.accounts.collateral_pool.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        usdc_required,
    )?;
    
    // Mint volatility tokens to user
    let token_mint_key = ctx.accounts.token_mint.key();
    let token_config_seeds = &[
        b"token_config", 
        token_mint_key.as_ref(),
        &[ctx.accounts.token_config.bump]
    ];
    let signer = &[&token_config_seeds[..]];
    
    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::MintTo {
                mint: ctx.accounts.token_mint.to_account_info(),
                to: ctx.accounts.user_token_account.to_account_info(),
                authority: ctx.accounts.token_config.to_account_info(),
            },
            signer,
        ),
        amount,
    )?;
    
    // Update token config state
    ctx.accounts.token_config.total_tokens_outstanding = ctx
        .accounts
        .token_config
        .total_tokens_outstanding
        .checked_add(amount)
        .ok_or(ContractError::MathOverflow)?;
    
    // Update or create user position
    let user_position = &mut ctx.accounts.user_position;
    
    // If the position is being created for the first time
    if user_position.owner == Pubkey::default() {
        user_position.owner = ctx.accounts.user.key();
        user_position.bump = ctx.bumps.user_position;
    }
    
    // Update position details
    user_position.entry_volatility = current_volatility;
    user_position.tokens_minted = user_position.tokens_minted
        .checked_add(amount)
        .ok_or(ContractError::MathOverflow)?;
    user_position.usdc_collateral = user_position.usdc_collateral
        .checked_add(usdc_required)
        .ok_or(ContractError::MathOverflow)?;
    user_position.mint_timestamp = Clock::get()?.unix_timestamp;
    
    Ok(())
}