use anchor_lang::prelude::*;

pub mod errors;
mod instructions;
pub mod state;
pub mod utils;

use instructions::*;

// rAREXWkxUP9Cr91tRVJ29NumDAEKvNpDWZNqcfSwBNG - program id
declare_id!("rAREXWkxUP9Cr91tRVJ29NumDAEKvNpDWZNqcfSwBNG");

#[program]
pub mod listings {
    use super::*;

    /// initializer a new market
    #[inline(always)]
    pub fn init_market(ctx: Context<InitMarket>) -> Result<()> {
        instructions::market::init::handler(ctx)
    }

    /// initializer a new market
    #[inline(always)]
    pub fn verify_mint(ctx: Context<VerifyMint>) -> Result<()> {
        instructions::market::verify_mint::handler(ctx)
    }

    /// initializer a new bid
    #[inline(always)]
    pub fn init_buy_order(ctx: Context<InitBuyOrder>, data: InitOrderData) -> Result<()> {
        instructions::order::init::buy::handler(ctx, data)
    }

    /// initializer a new listing
    #[inline(always)]
    pub fn init_sell_order<'info>(
        ctx: Context<'_, '_, '_, 'info, InitSellOrder<'info>>,
        data: InitOrderData,
    ) -> Result<()> {
        instructions::order::init::sell::handler(ctx, data)
    }

    /// edit a bid
    #[inline(always)]
    pub fn edit_buy_order(ctx: Context<EditBuyOrder>, data: EditBuyOrderData) -> Result<()> {
        instructions::order::edit::buy::handler(ctx, data)
    }

    /// edit a listing
    #[inline(always)]
    pub fn edit_sell_order(ctx: Context<EditSellOrder>, data: EditSellOrderData) -> Result<()> {
        instructions::order::edit::sell::handler(ctx, data)
    }

    /// fill a bid
    #[inline(always)]
    pub fn fill_buy_order<'info>(
        ctx: Context<'_, '_, '_, 'info, FillBuyOrder<'info>>,
    ) -> Result<()> {
        instructions::order::fill::buy::handler(ctx)
    }

    /// fill a listing
    #[inline(always)]
    pub fn fill_sell_order<'info>(
        ctx: Context<'_, '_, '_, 'info, FillSellOrder<'info>>,
    ) -> Result<()> {
        instructions::order::fill::sell::handler(ctx)
    }

    /// cancel a buy order
    #[inline(always)]
    pub fn close_buy_order(ctx: Context<CloseBuyOrder>) -> Result<()> {
        instructions::order::close::buy::handler(ctx)
    }

    /// cancel a sell order
    #[inline(always)]
    pub fn close_sell_order<'info>(
        ctx: Context<'_, '_, '_, 'info, CloseSellOrder<'info>>,
    ) -> Result<()> {
        instructions::order::close::sell::handler(ctx)
    }

    /*
        For placing bids on collections
    */
    /// initializer a new bidding wallet
    #[inline(always)]
    pub fn init_wallet(ctx: Context<InitBiddingWallet>, amount: u64) -> Result<()> {
        instructions::wallet::init::handler(ctx, amount)
    }

    /// edit a bidding wallet
    #[inline(always)]
    pub fn edit_wallet(
        ctx: Context<EditBiddingWallet>,
        amount_change: u64,
        increase: bool,
    ) -> Result<()> {
        instructions::wallet::edit::handler(ctx, amount_change, increase)
    }
}
