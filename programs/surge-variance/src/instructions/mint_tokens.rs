use crate::errors::ErrorCode;
use crate::state::*;
use crate::events::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

#[derive(Accounts)]
#[instruction(amount: u64, is_long: bool, epoch: u64, timestamp: i64, bumps: MarketBumps)]
pub struct MintTokens<'info> {
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

    pub token_program: Program<'info, Token>,
}

impl<'info> MintTokens<'info> {
    pub fn mint_tokens(ctx: Context<MintTokens>, amount: u64, is_long: bool, epoch: u64, timestamp: i64, bumps: MarketBumps) -> Result<()> {
        let market = &mut ctx.accounts.market;
        require!(!market.is_expired, ErrorCode::MarketExpired);

        // Transfer USDC from user to vault
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_usdc.to_account_info(),
                    to: ctx.accounts.usdc_vault.to_account_info(),
                    authority: ctx.accounts.user_authority.to_account_info(),
                },
            ),
            amount,
        )?;

        // Mint VAR tokens to user
        let epoch_bytes = epoch.to_le_bytes();
        let timestamp_bytes = timestamp.to_le_bytes();
        let seeds = &[
            b"market".as_ref(), 
            &epoch_bytes[..],
            &timestamp_bytes[..],
            &[bumps.market]
        ];
        let signer = &[&seeds[..]];

        if is_long {
            token::mint_to(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    token::MintTo {
                        mint: ctx.accounts.var_long_mint.to_account_info(),
                        to: ctx.accounts.user_var_long.to_account_info(),
                        authority: market.to_account_info(),
                    },
                    signer,
                ),
                amount,
            )?;
        } else {
            token::mint_to(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    token::MintTo {
                        mint: ctx.accounts.var_short_mint.to_account_info(),
                        to: ctx.accounts.user_var_short.to_account_info(),
                        authority: market.to_account_info(),
                    },
                    signer,
                ),
                amount,
            )?;
        }

        market.total_deposits = market
            .total_deposits
            .checked_add(amount)
            .ok_or(ErrorCode::NumberOverflow)?;

        // Emit tokens minted event
        emit!(TokensMinted {
            market: market.key(),
            user: ctx.accounts.user_authority.key(),
            amount,
            is_long,
            total_deposits: market.total_deposits,
        });

        Ok(())
    }
}
