use anchor_lang::prelude::*;

#[error_code]
pub enum MarketError {
    #[msg("Account passed in incorrectly")]
    WrongAccount,
}
