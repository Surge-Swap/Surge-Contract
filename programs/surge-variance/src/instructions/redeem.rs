use crate::errors::ErrorCode;
use crate::state::*;
use crate::events::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

#[derive(Accounts)]
#[instruction(epoch: u64, timestamp: i64, bumps: MarketBumps)]
pub struct Redeem<'info> {
    #[account(
        mut,
        seeds = [
            b"market", 
            &epoch.to_le_bytes()[..],
            &timestamp.to_le_bytes()[..],
        ],
        bump 
    )]
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
    /// CHECK: This account is not owned by this program, but we read from it
    pub volatility_stats: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
}

impl<'info> Redeem<'info> {
    pub fn redeem(ctx: Context<Redeem>, epoch: u64, timestamp: i64, bumps: MarketBumps) -> Result<()> {
        let market = &mut ctx.accounts.market;
        require!(!market.is_expired, ErrorCode::MarketExpired);

        // Get the annualized_volatility from the volatility_stats account
        let data = ctx.accounts.volatility_stats.try_borrow_data()?;
        if data.len() < 8 + 32 + 8 + 8 + 8 + 8 + 8 {
            return Err(ProgramError::InvalidAccountData.into());
        }
        
        let start_index = 8 + 32 + 8 + 8 + 8 + 8; // Offset to get to annualized_volatility
        let annualized_volatility_bytes = &data[start_index..start_index + 8];
        let annualized_volatility = f64::from_le_bytes(annualized_volatility_bytes.try_into().unwrap());

        // Calculate realized variance from the volatility
        let realized_variance = (annualized_volatility * 100.0) - (market.start_volatility * 100.0);
        if realized_variance < 0.0 {
            return Err(ErrorCode::NumberOverflow.into());
        }

        market.realized_variance = realized_variance;
        market.is_expired = true;

        // Calculate payouts
        let total_supply = market.total_deposits;
        let strike = market.strike;

        let long_payout = if realized_variance > strike {
            let variance_diff = realized_variance - strike;
            let scaled_diff = (variance_diff * (total_supply as f64) / 100.0) as u128;
            scaled_diff
        } else {
            0
        };

        let long_payout = u64::try_from(long_payout).map_err(|_| ErrorCode::NumberOverflow)?;

        let short_payout = total_supply
            .checked_sub(long_payout)
            .ok_or(ErrorCode::NumberOverflow)?;

        // Transfer payouts using the seeds for PDA signing
        let epoch_bytes = epoch.to_le_bytes();
        let timestamp_bytes = timestamp.to_le_bytes();
        let seeds = &[
            b"market".as_ref(), 
            &epoch_bytes[..],
            &timestamp_bytes[..],
            &[bumps.market]
        ];
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

        // Emit market redeemed event
        emit!(MarketRedeemed {
            market: market.key(),
            user: ctx.accounts.user_authority.key(),
            realized_variance,
            strike,
            long_payout,
            short_payout,
            total_deposits: total_supply,
        });

        Ok(())
    }
}
