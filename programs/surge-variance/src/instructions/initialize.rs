use crate::state::*;
use crate::events::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};

#[derive(Accounts)]
#[instruction(epoch: u64, strike: f64, timestamp: i64, bumps: MarketBumps)]
pub struct InitializeMarket<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 32 + 32 + 32 + 32 + 32 + 8 + 8 + 1 + 8 + 8 + 8 + 1 + 1 + 8,
        seeds = [
            b"market", 
            &epoch.to_le_bytes()[..],
            &timestamp.to_le_bytes()[..],
        ],
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
    /// CHECK: This account is not owned by this program, but we read from it
    pub volatility_stats: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitializeMarket<'info> {
    pub fn initialize_market(
        ctx: Context<InitializeMarket>,
        epoch: u64,
        strike: f64,
        timestamp: i64,
        bumps: MarketBumps,
    ) -> Result<()> {
        let market = &mut ctx.accounts.market;

        // Initialize market state
        market.epoch = epoch;
        market.strike = strike;
        market.timestamp = timestamp;
        market.authority = ctx.accounts.authority.key();
        market.usdc_vault = ctx.accounts.usdc_vault.key();
        market.var_long_mint = ctx.accounts.var_long_mint.key();
        market.var_short_mint = ctx.accounts.var_short_mint.key();
        market.volatility_stats = ctx.accounts.volatility_stats.key();
        market.bumps = bumps;
        market.is_initialized = true;
        market.is_expired = false;
        market.total_deposits = 0;
        
        // Get the annualized_volatility from the volatility_stats account
        let data = ctx.accounts.volatility_stats.try_borrow_data()?;
        if data.len() < 8 + 32 + 8 + 8 + 8 + 8 + 8 {
            return Err(ProgramError::InvalidAccountData.into());
        }
        
        let start_index = 8 + 32 + 8 + 8 + 8 + 8; // Offset to get to annualized_volatility
        let annualized_volatility_bytes = &data[start_index..start_index + 8];
        let annualized_volatility = f64::from_le_bytes(annualized_volatility_bytes.try_into().unwrap());
        
        market.start_volatility = annualized_volatility;

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

        // Emit market initialized event
        emit!(MarketInitialized {
            market: market.key(),
            authority: market.authority,
            usdc_vault: market.usdc_vault,
            var_long_mint: market.var_long_mint,
            var_short_mint: market.var_short_mint,
            epoch: market.epoch,
            strike: market.strike,
            timestamp: market.timestamp,
            start_volatility: market.start_volatility,
        });

        Ok(())
    }
}
