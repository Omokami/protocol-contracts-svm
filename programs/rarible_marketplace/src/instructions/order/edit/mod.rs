use anchor_lang::prelude::*;

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy)]
pub struct EditBuyOrderData {
    pub new_size: u64,
    pub new_price: u64,
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy)]
pub struct EditSellOrderData {
    pub new_price: u64,
}

pub mod buy;
pub mod sell;

pub use buy::*;
pub use sell::*;
