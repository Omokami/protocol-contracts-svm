use anchor_lang::prelude::*;

#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct InitOrderData {
    pub nonce: Pubkey,
    pub price: u64,
    pub size: u64,
}

pub mod buy;
pub mod sell;

pub use buy::*;
pub use sell::*;
