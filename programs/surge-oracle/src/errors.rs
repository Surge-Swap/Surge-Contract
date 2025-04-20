use anchor_lang::prelude::*;

#[error_code]
pub enum OracleError {
    #[msg("Invalid Pyth price account")]
    InvalidPythAccount,

    #[msg("No price is available from Pyth")]
    NoPriceAvailable,

    #[msg("Invalid price data")]
    InvalidPriceData,

    #[msg("Invalid authority")]
    InvalidAuthority,
}
