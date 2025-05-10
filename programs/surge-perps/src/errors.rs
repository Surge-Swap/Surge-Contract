use anchor_lang::prelude::*;

#[error_code]
pub enum PerpError {
    #[msg("Position already open")]
    ExistingPosition,
    #[msg("No active position")]
    NoActivePosition,
    #[msg("Insufficient margin")]
    BadMargin,
    #[msg("Oracle stale / unavailable")]
    OracleStale,
    #[msg("Invalid vault. The provided vault does not match the configured vault.")]
    InvalidVault,
    #[msg("Insufficient tokens to burn")]
    InsufficientTokens,
    #[msg("Insufficient vault balance")]
    InsufficientVaultBalance,
    #[msg("Position already active")]
    PositionAlreadyExists,
}