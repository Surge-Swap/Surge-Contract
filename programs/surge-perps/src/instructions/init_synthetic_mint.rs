use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};
use crate::state::SyntheticMint;

#[derive(Accounts)]
pub struct InitSyntheticMint<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        seeds = [b"synthetic_mint_token"],
        bump,
        mint::decimals = 6,
        mint::authority = synthetic_mint_token,
        mint::freeze_authority = synthetic_mint_token,
    )]
    pub synthetic_mint_token: Account<'info, Mint>,

    #[account(
        init,
        payer = payer,
        space = SyntheticMint::LEN,
        seeds = [b"synthetic_mint"],
        bump,
    )]
    pub synthetic_mint: Account<'info, SyntheticMint>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> InitSyntheticMint<'info> {
    pub fn init_synthetic_mint(ctx: Context<InitSyntheticMint>) -> Result<()> {
        // Store the bump in the synthetic mint account
        ctx.accounts.synthetic_mint.bump = ctx.bumps.synthetic_mint;
        Ok(())
    }
}