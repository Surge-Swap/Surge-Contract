use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer, Burn};

use crate::{state::*, errors::ContractError};

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct RedeemTokens<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    /// USDC token account to receive funds
    #[account(
        mut,
        constraint = user_usdc_account.owner == user.key() @ ContractError::Unauthorized,
        constraint = user_usdc_account.mint == token_config.usdc_mint @ ContractError::InvalidOracleData,
    )]
    pub user_usdc_account: Account<'info, TokenAccount>,
    
    /// Volatility token account of the user
    #[account(
        mut,
        constraint = user_token_account.owner == user.key() @ ContractError::Unauthorized,
        constraint = user_token_account.mint == token_mint.key() @ ContractError::InvalidOracleData,
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    
    /// Fee destination account
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
        constraint = collateral_pool.mint == token_config.usdc_mint @ ContractError::InvalidOracleData,
    )]
    pub collateral_pool: Account<'info, TokenAccount>,
    
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
        mut,
        seeds = [b"user_position", user.key().as_ref(), token_mint.key().as_ref()],
        bump = user_position.bump,
        constraint = user_position.owner == user.key() @ ContractError::Unauthorized,
    )]
    pub user_position: Account<'info, UserPosition>,
    
    /// Oracle account with volatility data
    /// CHECK: Account is validated through VolatilityStats::load_from_account_info
    pub oracle: AccountInfo<'info>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn redeem_tokens(ctx: Context<RedeemTokens>, amount: u64) -> Result<()> {
    // Validate amount is greater than 0
    require!(amount > 0, ContractError::InvalidAmount);
    
    // Check that user has enough tokens in their account
    require!(
        ctx.accounts.user_token_account.amount >= amount,
        ContractError::InsufficientTokens
    );
    
    // Check that user position has enough tokens
    require!(
        ctx.accounts.user_position.tokens_minted >= amount,
        ContractError::InsufficientTokens
    );
    
    // Get current volatility from oracle
    let current_volatility = VolatilityStats::load_from_account_info(&ctx.accounts.oracle)?;
    let entry_volatility = ctx.accounts.user_position.entry_volatility;
    
    msg!("Entry volatility: {}", entry_volatility);
    msg!("Current volatility: {}", current_volatility);
    
    // Calculate redemption value based on volatility change and token amount
    let usdc_per_vol = ctx.accounts.token_config.usdc_per_vol_point;
    
    // Convert volatilities to basis points (multiply by 1000 for precision)
    let entry_vol_points = (entry_volatility * 1000.0) as u64;
    let current_vol_points = (current_volatility * 1000.0) as u64;
    
    // Calculate redemption amount
    let redemption_value = calculate_redemption_value(
        amount,
        entry_vol_points,
        current_vol_points,
        usdc_per_vol,
    )?;
    
    msg!("Redemption value: {}", redemption_value);
    
    // Calculate fee
    let fee_amount = redemption_value
        .checked_mul(ctx.accounts.token_config.fee_bps as u64)
        .ok_or(ContractError::MathOverflow)?
        .checked_div(10000)
        .ok_or(ContractError::MathOverflow)?;
    
    msg!("Fee amount: {}", fee_amount);
    
    // Final amount after fee
    let final_amount = redemption_value
        .checked_sub(fee_amount)
        .ok_or(ContractError::MathOverflow)?;
    
    // Ensure pool has enough USDC to pay out
    require!(
        ctx.accounts.collateral_pool.amount >= final_amount,
        ContractError::InsufficientBalance
    );
    
    // Burn the volatility tokens
    token::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.token_mint.to_account_info(),
                from: ctx.accounts.user_token_account.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        amount,
    )?;
    
    // Transfer fee to fee destination
    let token_mint_key = ctx.accounts.token_mint.key();
    let token_config_seeds = &[
        b"token_config", 
        token_mint_key.as_ref(),
        &[ctx.accounts.token_config.bump]
    ];
    let signer = &[&token_config_seeds[..]];
    
    // Only transfer fee if it's greater than 0
    if fee_amount > 0 {
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.collateral_pool.to_account_info(),
                    to: ctx.accounts.fee_destination.to_account_info(),
                    authority: ctx.accounts.token_config.to_account_info(),
                },
                signer,
            ),
            fee_amount,
        )?;
    }
    
    // Transfer USDC to user
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.collateral_pool.to_account_info(),
                to: ctx.accounts.user_usdc_account.to_account_info(),
                authority: ctx.accounts.token_config.to_account_info(),
            },
            signer,
        ),
        final_amount,
    )?;
    
    // Update token config state
    ctx.accounts.token_config.total_tokens_outstanding = ctx
        .accounts
        .token_config
        .total_tokens_outstanding
        .checked_sub(amount)
        .ok_or(ContractError::MathOverflow)?;
    
    // Update user position
    ctx.accounts.user_position.tokens_minted = ctx
        .accounts
        .user_position
        .tokens_minted
        .checked_sub(amount)
        .ok_or(ContractError::MathOverflow)?;
    
    // Scale down the collateral proportionally
    let collateral_reduction = ctx
        .accounts
        .user_position
        .usdc_collateral
        .checked_mul(amount)
        .ok_or(ContractError::MathOverflow)?
        .checked_div(ctx.accounts.user_position.tokens_minted + amount) // Add back amount to get original total
        .unwrap_or(ctx.accounts.user_position.usdc_collateral);
    
    ctx.accounts.user_position.usdc_collateral = ctx
        .accounts
        .user_position
        .usdc_collateral
        .checked_sub(collateral_reduction)
        .ok_or(ContractError::MathOverflow)?;
    
    Ok(())
}

// Helper function to calculate redemption value
fn calculate_redemption_value(
    amount: u64,
    entry_vol_points: u64,
    current_vol_points: u64,
    usdc_per_vol: u64,
) -> Result<u64> {
    // Base redemption is tokens * entry_volatility * usdc_per_vol_point
    let base_value = amount
        .checked_mul(entry_vol_points)
        .ok_or(ContractError::MathOverflow)?
        .checked_mul(usdc_per_vol)
        .ok_or(ContractError::MathOverflow)?
        .checked_div(1000) // Adjust for the volatility scaling
        .ok_or(ContractError::MathOverflow)?;
    
    // Calculate profit/loss based on volatility change
    if current_vol_points > entry_vol_points {
        // Volatility increased, user profits
        let profit_per_vol_point = usdc_per_vol
            .checked_mul(amount)
            .ok_or(ContractError::MathOverflow)?;
        
        let vol_diff = current_vol_points
            .checked_sub(entry_vol_points)
            .ok_or(ContractError::MathOverflow)?;
        
        let profit = profit_per_vol_point
            .checked_mul(vol_diff)
            .ok_or(ContractError::MathOverflow)?
            .checked_div(1000) // Adjust for the volatility scaling
            .ok_or(ContractError::MathOverflow)?;
        
        base_value
            .checked_add(profit)
            .ok_or(ContractError::MathOverflow.into())
    } else if current_vol_points < entry_vol_points {
        // Volatility decreased, user takes a loss
        let loss_per_vol_point = usdc_per_vol
            .checked_mul(amount)
            .ok_or(ContractError::MathOverflow)?;
        
        let vol_diff = entry_vol_points
            .checked_sub(current_vol_points)
            .ok_or(ContractError::MathOverflow)?;
        
        let loss = loss_per_vol_point
            .checked_mul(vol_diff)
            .ok_or(ContractError::MathOverflow)?
            .checked_div(1000) // Adjust for the volatility scaling
            .ok_or(ContractError::MathOverflow)?;
        
        // Ensure loss doesn't exceed base value
        if loss >= base_value {
            // Return minimum value (1) to avoid complete loss
            Ok(1)
        } else {
            base_value
                .checked_sub(loss)
                .ok_or(ContractError::MathOverflow.into())
        }
    } else {
        // No change in volatility
        Ok(base_value)
    }
}

