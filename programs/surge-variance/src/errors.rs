use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Market is already expired")]
    MarketExpired,

    #[msg("Numeric overflow occurred")]
    NumberOverflow,
}
