use crate::state::*;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = VolatilityStats::SIZE
    )]
    pub volatility_stats: Account<'info, VolatilityStats>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

impl Initialize<'_> {
    pub fn initialize_volatility_stats(ctx: Context<Initialize>) -> Result<()> {
        let stats = &mut ctx.accounts.volatility_stats;
        stats.update_volatility(
            Some(0),   // last_price
            Some(0.0), // mean
            Some(0.0), // m2
            Some(0),   // count
            Some(0.0), // annualized_volatility
        );
        stats.authority = ctx.accounts.authority.key();

        msg!("Volatility stats account initialized with Welford's method");
        Ok(())
    }
}
