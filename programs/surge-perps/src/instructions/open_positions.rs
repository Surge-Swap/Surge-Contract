use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::errors::PerpError;
use crate::events::PositionOpened;


#[derive(Accounts)]
#[instruction(direction: Side)]
pub struct OpenPosition<'info> {
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
        init,
        payer = owner,
        space = Position::LEN,
        seeds = [b"position", owner.key().as_ref()],
        bump
    )]
    pub position: Account<'info, Position>,

    pub volatility_stats: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    
    pub system_program: Program<'info, System>,
}

impl<'info> OpenPosition<'info> {
    pub fn open_position(
        ctx: Context<OpenPosition>,
        direction: Side,
        margin: u64,
    ) -> Result<()> {
        require!(margin > 0, PerpError::BadMargin);

        let entry_vol = VolatilityStats::load_from_account_info(&ctx.accounts.volatility_stats)?;
        let timestamp = Clock::get()?.unix_timestamp;

        let pos = &mut ctx.accounts.position;
        pos.owner            = ctx.accounts.owner.key();
        pos.direction        = direction;
        pos.entry_vol        = entry_vol;
        pos.size             = margin;
        pos.margin           = margin;
        pos.bump             = ctx.bumps.position;
        pos.created_at       = timestamp;
        pos.is_active        = true;

        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from:   ctx.accounts.user_usdc.to_account_info(),
                to:     ctx.accounts.vault.to_account_info(),
                authority: ctx.accounts.owner.to_account_info(),
            },
        );
        token::transfer(cpi_ctx, margin)?;

        emit!(PositionOpened {
            owner: ctx.accounts.owner.key(),
            position: ctx.accounts.position.key(),
            direction,
            entry_vol,
            size: margin,
            margin,
            timestamp,
        });

        Ok(())
    }
}
