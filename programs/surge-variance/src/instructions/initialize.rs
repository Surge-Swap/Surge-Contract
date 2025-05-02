use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};

#[derive(Accounts)]
#[instruction(epoch: u64, strike: u64, bumps: MarketBumps)]
pub struct InitializeMarket<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 32 + 32 + 32 + 32 + 32 + 8 + 1 + 8 + 8 + 8 + 1 + 1 + 8,
        seeds = [b"market"],
        bump
    )]
    pub market: Account<'info, Market>,

    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut)]
    pub usdc_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub var_long_mint: Account<'info, Mint>,

    #[account(mut)]
    pub var_short_mint: Account<'info, Mint>,

    /// The volatility stats account from the oracle program
    pub volatility_stats: Account<'info, VolatilityStats>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitializeMarket<'info> {
    pub fn initialize_market(
        ctx: Context<InitializeMarket>,
        epoch: u64,
        strike: u64,
        bumps: MarketBumps,
    ) -> Result<()> {
        let market = &mut ctx.accounts.market;

        // Initialize market state
        market.epoch = epoch;
        market.strike = strike;
        market.authority = ctx.accounts.authority.key();
        market.usdc_vault = ctx.accounts.usdc_vault.key();
        market.var_long_mint = ctx.accounts.var_long_mint.key();
        market.var_short_mint = ctx.accounts.var_short_mint.key();
        market.volatility_stats = ctx.accounts.volatility_stats.key();
        market.bumps = bumps;
        market.is_initialized = true;
        market.is_expired = false;
        market.total_deposits = 0;
        market.start_volatility = ctx.accounts.volatility_stats.annualized_volatility;

        // Transfer authority of the mints to the PDA
        token::set_authority(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::SetAuthority {
                    current_authority: ctx.accounts.authority.to_account_info(),
                    account_or_mint: ctx.accounts.var_long_mint.to_account_info(),
                },
            ),
            token::spl_token::instruction::AuthorityType::MintTokens,
            Some(market.key()),
        )?;

        token::set_authority(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::SetAuthority {
                    current_authority: ctx.accounts.authority.to_account_info(),
                    account_or_mint: ctx.accounts.var_short_mint.to_account_info(),
                },
            ),
            token::spl_token::instruction::AuthorityType::MintTokens,
            Some(market.key()),
        )?;

        Ok(())
    }
}
