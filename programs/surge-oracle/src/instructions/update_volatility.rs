use anchor_lang::prelude::*;
use pyth_solana_receiver_sdk::price_update::{get_feed_id_from_hex, PriceUpdateV2};

use crate::{errors::OracleError, state::VolatilityStats};

#[derive(Accounts)]
pub struct UpdateVolatility<'info> {
    #[account(
        mut,
        has_one = authority,
    )]
    pub volatility_stats: Account<'info, VolatilityStats>,

    pub authority: Signer<'info>,

    pub price_update: Account<'info, PriceUpdateV2>,
}

impl UpdateVolatility<'_> {
    pub fn update_volatility(ctx: Context<UpdateVolatility>) -> Result<()> {
        let stats = &mut ctx.accounts.volatility_stats;
        let price_update = &ctx.accounts.price_update;
        let max_age = 30;

        let feed_id: [u8; 32] =
            get_feed_id_from_hex("7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE")?;
        let price = price_update
            .get_price_no_older_than(&Clock::get()?, max_age, &feed_id)
            .map_err(|_| error!(OracleError::NoPriceAvailable))?;

        msg!(
            "Current SOL/USD price: ({} ± {}) * 10^{}",
            price.price,
            price.conf,
            price.exponent
        );

        let current_price_raw = (price.price as f64) * 10f64.powi(price.exponent);
        let current_price = (current_price_raw * 1_000_000.0) as u64;

        let (mut new_mean, mut new_m2, mut new_count, mut new_annualized_volatility) = (
            stats.mean,
            stats.m2,
            stats.count,
            stats.annualized_volatility,
        );

        if stats.count > 0 {
            let last_price_float = (stats.last_price as f64) / 1_000_000.0;
            let log_return = (current_price_raw / last_price_float).ln();
            let delta = log_return - stats.mean;
            new_count += 1;
            new_mean += delta / (new_count as f64);
            new_m2 += delta * (log_return - new_mean);

            if new_count > 1 {
                let variance = new_m2 / ((new_count - 1) as f64);
                let daily_vol = variance.sqrt();
                new_annualized_volatility = daily_vol * (252.0_f64).sqrt();
                msg!(
                    "Updated annualized volatility (Welford): {}",
                    new_annualized_volatility
                );
            }
        } else {
            new_count = 1;
        }

        stats.update_volatility(
            Some(current_price),
            Some(new_mean),
            Some(new_m2),
            Some(new_count),
            Some(new_annualized_volatility),
        );

        Ok(())
    }
}
