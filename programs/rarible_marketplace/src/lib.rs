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
    pub fn bid(ctx: Context<BidNft>, data: BidData) -> Result<()> {
        instructions::order::bid::handler(ctx, data)
    }

    /// initializer a new listing
    #[inline(always)]
    pub fn list<'info>(
        ctx: Context<'_, '_, '_, 'info, ListNft<'info>>,
        data: ListData,
    ) -> Result<()> {
        instructions::order::list::handler(ctx, data)
    }

    /// fill a listing
    #[inline(always)]
    pub fn fill_order<'info>(
        ctx: Context<'_, '_, '_, 'info, FillOrder<'info>>,
    ) -> Result<()> {
        instructions::order::fill::handler(ctx)
    }

    /// cancel a buy order
    #[inline(always)]
    pub fn cancel_bid(ctx: Context<CancelBid>) -> Result<()> {
        instructions::order::cancel_bid::handler(ctx)
    }

    /// cancel a sell order
    #[inline(always)]
    pub fn cancel_listing<'info>(
        ctx: Context<'_, '_, '_, 'info, CancelListing<'info>>,
    ) -> Result<()> {
        instructions::order::cancel_list::handler(ctx)
    }
}
