use crate::errors::ErrorCode;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct Redeem<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,

    pub user_authority: Signer<'info>,

    #[account(mut)]
    pub user_usdc: Account<'info, TokenAccount>,

    #[account(mut)]
    pub usdc_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub var_long_mint: Account<'info, Mint>,

    #[account(mut)]
    pub var_short_mint: Account<'info, Mint>,

    #[account(mut)]
    pub user_var_long: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_var_short: Account<'info, TokenAccount>,

    /// The volatility stats account from the oracle program
    /// CHECK: This account is actually a VolatilityStats account from the oracle program
    pub volatility_stats: Account<'info, VolatilityStats>,

    pub token_program: Program<'info, Token>,
}

impl<'info> Redeem<'info> {
    pub fn redeem(ctx: Context<Redeem>) -> Result<()> {
        let market = &mut ctx.accounts.market;
        require!(!market.is_expired, ErrorCode::MarketExpired);

        // Get realized variance from the volatility oracle
        let volatility_stats = &ctx.accounts.volatility_stats;
        let realized_variance = ((volatility_stats.annualized_volatility * 100.0) as u64)
            .checked_sub((market.start_volatility * 100.0) as u64)
            .ok_or(ErrorCode::NumberOverflow)?;

        market.realized_variance = realized_variance;
        market.is_expired = true;

        // Calculate payouts
        let total_supply = market.total_deposits;
        let strike = market.strike;

        let long_payout = if realized_variance > strike {
            ((realized_variance - strike) as u128)
                .checked_mul(total_supply as u128)
                .ok_or(ErrorCode::NumberOverflow)?
                .checked_div(100)
                .ok_or(ErrorCode::NumberOverflow)?
        } else {
            0
        };

        let long_payout = u64::try_from(long_payout).map_err(|_| ErrorCode::NumberOverflow)?;

        let short_payout = total_supply
            .checked_sub(long_payout)
            .ok_or(ErrorCode::NumberOverflow)?;

        // Transfer payouts
        let seeds = &[b"market".as_ref(), &[market.bumps.market]];
        let signer = &[&seeds[..]];

        if long_payout > 0 {
            token::transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.usdc_vault.to_account_info(),
                        to: ctx.accounts.user_usdc.to_account_info(),
                        authority: market.to_account_info(),
                    },
                    signer,
                ),
                long_payout,
            )?;
        }

        if short_payout > 0 {
            token::transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.usdc_vault.to_account_info(),
                        to: ctx.accounts.user_usdc.to_account_info(),
                        authority: market.to_account_info(),
                    },
                    signer,
                ),
                short_payout,
            )?;
        }

        // Burn the VAR tokens
        token::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Burn {
                    mint: ctx.accounts.var_long_mint.to_account_info(),
                    from: ctx.accounts.user_var_long.to_account_info(),
                    authority: ctx.accounts.user_authority.to_account_info(),
                },
            ),
            ctx.accounts.user_var_long.amount,
        )?;

        token::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Burn {
                    mint: ctx.accounts.var_short_mint.to_account_info(),
                    from: ctx.accounts.user_var_short.to_account_info(),
                    authority: ctx.accounts.user_authority.to_account_info(),
                },
            ),
            ctx.accounts.user_var_short.amount,
        )?;

        Ok(())
    }
}
