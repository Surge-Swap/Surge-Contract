use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::state::{Position, Side, VolatilityStats};
use crate::errors::PerpError;
use crate::events::PositionClosed;

#[derive(Accounts)]
pub struct ClosePosition<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub user_usdc: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"vault"],
        bump
    )]
    pub vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"position", owner.key().as_ref()],
        bump,
        has_one = owner,
        constraint = position.is_active @ PerpError::NoActivePosition
    )]
    pub position: Account<'info, Position>,

   
    pub volatility_stats: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
}

impl<'info> ClosePosition<'info> {
    pub fn close_position(ctx: Context<ClosePosition>) -> Result<()> {
        let current_vol = VolatilityStats::load_from_account_info(&ctx.accounts.volatility_stats)?;
        let timestamp = Clock::get()?.unix_timestamp;
        
        let position_key = ctx.accounts.position.key();
        let owner_key = ctx.accounts.owner.key();
        
        let pos = &mut ctx.accounts.position;
        let direction = pos.direction;
        let entry_vol = pos.entry_vol;
        let size = pos.size;
        let margin = pos.margin;

        let delta = current_vol - entry_vol;
        let pnl: f64 = match direction {
            Side::Long  => delta * size as f64,
            Side::Short => -delta * size as f64,
        };

        let pnl_lamports: i64 = (pnl) as i64;

        let mut pay = margin as i64 + pnl_lamports;
        if pay < 0 {
            pay = 0;
        }

        let payout = pay as u64;

        if payout > 0 {
            let vault_bump = ctx.bumps.vault;
            let vault_seeds = [b"vault".as_ref(), &[vault_bump]];
            let signer = &[&vault_seeds[..]];
            let cpi_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.vault.to_account_info(),
                    to:   ctx.accounts.user_usdc.to_account_info(),
                    authority: ctx.accounts.vault.to_account_info(),
                },
                signer,
            );
            token::transfer(cpi_ctx, payout)?;
        }

        pos.is_active = false;
        
        emit!(PositionClosed {
            owner: owner_key,
            position: position_key,
            direction,
            entry_vol,
            exit_vol: current_vol,
            size,
            margin,
            pnl: pnl_lamports,
            payout,
            timestamp,
        });

        Ok(())
    }
}
