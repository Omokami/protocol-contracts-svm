use anchor_lang::prelude::*;

pub mod errors;
mod instructions;
pub mod state;
pub mod utils;

use instructions::*;

// rAREXWkxUP9Cr91tRVJ29NumDAEKvNpDWZNqcfSwBNG - program id
declare_id!("B6ckscvApoZpBZ7cKGYa4VK4vTc1x9XjPk6osKfK7rSZ");

#[program]
pub mod marketplace {

    use super::*;

    /// initializer a new market
    #[inline(never)]
    pub fn init_market(ctx: Context<InitMarket>) -> Result<()> {
        instructions::market::init::handler(ctx)
    }

    /// initializer a new market
    #[inline(never)]
    pub fn verify_mint(ctx: Context<VerifyMint>) -> Result<()> {
        instructions::market::verify_mint::handler(ctx)
    }

    /// initializer a new bid
    #[inline(never)]
    pub fn bid(ctx: Context<BidNft>, data: BidData) -> Result<()> {
        instructions::order::bid::handler(ctx, data)
    }

    /// initializer a new listing
    #[inline(never)]
    pub fn list<'info>(
        ctx: Context<'_, '_, '_, 'info, ListNft<'info>>,
        data: ListData,
    ) -> Result<()> {
        instructions::order::list::handler(ctx, data)
    }

    /// fill a listing
    #[inline(never)]
    pub fn fill_order<'info>(
        ctx: Context<'_, '_, '_, 'info, FillOrder<'info>>,
    ) -> Result<()> {
        instructions::order::fill::handler(ctx)
    }

    /// cancel a buy order
    #[inline(never)]
    pub fn cancel_bid(ctx: Context<CancelBid>) -> Result<()> {
        instructions::order::cancel_bid::handler(ctx)
    }

    /// cancel a sell order
    #[inline(never)]
    pub fn cancel_listing<'info>(
        ctx: Context<'_, '_, '_, 'info, CancelListing<'info>>,
    ) -> Result<()> {
        instructions::order::cancel_list::handler(ctx)
    }
}
