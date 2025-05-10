use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use crate::state::VaultConfig;

#[derive(Accounts)]
pub struct SetVault<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init_if_needed,
        payer = authority,
        space = VaultConfig::LEN,
        seeds = [b"vault_config"],
        bump,
    )]
    pub vault_config: Account<'info, VaultConfig>,

    /// The token account to use as the vault
    pub custom_vault: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
}

impl<'info> SetVault<'info> {
    pub fn set_vault(ctx: Context<SetVault>) -> Result<()> {
        let vault_config = &mut ctx.accounts.vault_config;
        vault_config.custom_vault = ctx.accounts.custom_vault.key();
        vault_config.bump = ctx.bumps.vault_config;
        
        msg!("Vault config initialized with custom vault: {}", vault_config.custom_vault);
        Ok(())
    }
} 