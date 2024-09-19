use anchor_lang::prelude::*;

#[error_code]
pub enum MarketError {
    #[msg("Account passed in incorrectly")]
    WrongAccount,
    #[msg("Amount overflow")]
    AmountOverflow,
    #[msg("Amount underflow")]
    AmountUnderflow,
}
