use anchor_lang::prelude::*;

use crate::state::*;

#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct BidData {
    pub nonce: Pubkey,
    pub price: u64,
    pub size: u64,
}

#[derive(Accounts)]
#[instruction(data: BidData)]
#[event_cpi]
pub struct BidNft<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    #[account(
        constraint = Market::is_active(market.state),
        seeds = [MARKET_SEED,
        market.market_identifier.as_ref()],
        bump,
    )]
    pub market: Box<Account<'info, Market>>,
    #[account(
        constraint = data.price > 0 && data.size > 0,
        init,
        seeds = [ORDER_SEED,
        data.nonce.as_ref(),
        market.key().as_ref(),
        initializer.key().as_ref()],
        bump,
        payer = initializer,
        space = 8 + std::mem::size_of::<Order>()
    )]
    pub order: Box<Account<'info, Order>>,
    /// CHECK: can be anything
    pub nft_mint: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[inline(always)]
pub fn handler(ctx: Context<BidNft>, data: BidData) -> Result<()> {
    msg!("Initialize a new buy order: {}", ctx.accounts.order.key());

    let clock = Clock::get()?;

    // Transfer bid funds TODO;
    
    // create a new order with size 1
    Order::init(
        &mut ctx.accounts.order,
        ctx.accounts.market.key(),
        ctx.accounts.initializer.key(),
        data.nonce,
        ctx.accounts.nft_mint.key(),
        clock.unix_timestamp,
        OrderSide::Buy.into(),
        data.size,
        data.price,
        OrderState::Ready.into(),
        true,
    );

    emit_cpi!(Order::get_edit_event(
        &mut ctx.accounts.order.clone(),
        ctx.accounts.order.key(),
        ctx.accounts.market.market_identifier,
        OrderEditType::Init,
    ));

    Ok(())
}
