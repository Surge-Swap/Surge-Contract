use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint, MintTo};   // +Mint, MintTo
use anchor_spl::associated_token::AssociatedToken;                            // NEW

use crate::errors::PerpError;
use crate::events::PositionOpened;


#[derive(Accounts)]
#[instruction(direction: Side)]
pub struct OpenPosition<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub user_usdc: Account<'info, TokenAccount>,

    // The vault - can be either a custom token account or the PDA
    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"position", owner.key().as_ref()],
        bump
    )]
    pub position: Account<'info, Position>,

    #[account(
        mut,
        seeds = [b"synthetic_mint_token"],
        bump,
    )]
    pub synthetic_mint: Account<'info, Mint>,

    /// User's ATA for vVOL (auto-creates if missing)
    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = synthetic_mint,
        associated_token::authority = owner,
    )]
    pub user_vvol: Account<'info, TokenAccount>,

    /// CHECK: This account stores volatility stats which are loaded and validated in the instruction logic
    pub volatility_stats: AccountInfo<'info>,

    /// OPTIONAL: Vault config account that can specify a custom vault token account
    /// If provided, we'll verify that the passed vault matches this config
    #[account(
        seeds = [b"vault_config"],
        bump,
    )]
    pub vault_config: Option<Account<'info, VaultConfig>>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,   
    pub system_program: Program<'info, System>,
}

impl<'info> OpenPosition<'info> {
    pub fn open_position(
        ctx: Context<OpenPosition>,
        direction: Side,
        margin: u64,
    ) -> Result<()> {
        require!(margin > 0, PerpError::BadMargin);

        // If vault_config is provided, verify that vault matches config
        if let Some(vault_config) = &ctx.accounts.vault_config {
            require!(
                vault_config.custom_vault == ctx.accounts.vault.key(), 
                PerpError::InvalidVault
            );
        }
      
        let entry_vol = VolatilityStats::load_from_account_info(&ctx.accounts.volatility_stats)?;
        let timestamp = Clock::get()?.unix_timestamp;

        // Check if the account is a newly created one or reused inactive one
        let account_info = ctx.accounts.position.to_account_info();
        let is_new_account = account_info.data_is_empty();
        
        // If the account exists and is active, return an error
        if !is_new_account && ctx.accounts.position.is_active {
            return Err(PerpError::PositionAlreadyExists.into());
        }
        
        // If it's a new account, initialize it
        if is_new_account {
            // Create the position account
            let space = Position::LEN;
            let lamports = Rent::get()?.minimum_balance(space);
            let position_key = ctx.accounts.position.key();
            let owner_key = ctx.accounts.owner.key();
            let position_bump = ctx.bumps.position;
            
            let create_ix = anchor_lang::solana_program::system_instruction::create_account(
                &owner_key,
                &position_key,
                lamports,
                space as u64,
                ctx.program_id,
            );
            
            anchor_lang::solana_program::program::invoke_signed(
                &create_ix,
                &[
                    ctx.accounts.owner.to_account_info(),
                    ctx.accounts.position.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                ],
                &[&[
                    b"position", 
                    owner_key.as_ref(),
                    &[position_bump]
                ]],
            )?;
        }

        // Now we can safely mutate the position account
        let pos = &mut ctx.accounts.position;
        
        // ----- 2. write Position PDA -----
        pos.owner      = ctx.accounts.owner.key();
        pos.direction  = direction;
        pos.entry_vol  = entry_vol;
        pos.size       = margin;       // 1:1 notional for MVP
        pos.margin     = margin;
        pos.bump       = ctx.bumps.position;
        pos.created_at = timestamp;
        pos.is_active  = true;

        // ----- 3. move USDC collateral into vault -----
        let cpi_usdc = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_usdc.to_account_info(),
                to:   ctx.accounts.vault.to_account_info(),
                authority: ctx.accounts.owner.to_account_info(),
            },
        );
        token::transfer(cpi_usdc, margin)?;

        // ----- 4. mint vVOL to user ATA -----
        let synthetic_bump = ctx.bumps.synthetic_mint;
        let seeds: &[&[u8]] = &[b"synthetic_mint_token".as_ref(), &[synthetic_bump]];
        let signer_seeds = &[&seeds[..]];

        let cpi_mint = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint:      ctx.accounts.synthetic_mint.to_account_info(),
                to:        ctx.accounts.user_vvol.to_account_info(),
                authority: ctx.accounts.synthetic_mint.to_account_info(),
            },
            signer_seeds,
        );
        token::mint_to(cpi_mint, pos.size)?;

        // ----- 5. emit event -----
        emit!(PositionOpened {
            owner:     ctx.accounts.owner.key(),
            position:  ctx.accounts.position.key(),
            direction,
            entry_vol,
            size:      margin,
            margin,
            timestamp,
        });

        Ok(())
    }
}
