use anchor_lang::prelude::*;

#[error_code]
pub enum ContractError {
    #[msg("Oracle account data is stale or invalid")]
    OracleStale,
    
    #[msg("Insufficient USDC balance")]
    InsufficientBalance,
    
    #[msg("Invalid fee percentage, must be between 0 and 10000")]
    InvalidFeePercentage,
    
    #[msg("Only authority can perform this action")]
    Unauthorized,
    
    #[msg("Invalid token amount")]
    InvalidAmount,
    
    #[msg("Math overflow")]
    MathOverflow,
    
    #[msg("Invalid oracle data or account mismatch")]
    InvalidOracleData,
    
    #[msg("Position not found")]
    PositionNotFound,
    
    #[msg("Insufficient tokens to redeem")]
    InsufficientTokens,
}