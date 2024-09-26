use anchor_lang::prelude::*;

use crate::state::*;

#[derive(Accounts)]
#[instruction()]
#[event_cpi]
pub struct CancelBid<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    #[account(
        mut,
        constraint = order.owner == initializer.key(),
        constraint = Order::is_active(order.state),
        // constraint = order.side == OrderSide::Buy.into() || order.side == OrderSide::CompressedBuy.into(),
        seeds = [ORDER_SEED,
        order.nonce.as_ref(),
        order.market.as_ref(),
        initializer.key().as_ref()],
        bump,
        close = initializer,
    )]
    pub order: Box<Account<'info, Order>>,
    #[account(
        constraint = market.key() == order.market,
        seeds = [MARKET_SEED,
        market.market_identifier.as_ref()],
        bump,
    )]
    pub market: Box<Account<'info, Market>>,
}

#[inline(always)]
pub fn handler(ctx: Context<CancelBid>) -> Result<()> {
    msg!("Close buy order account: {}", ctx.accounts.order.key());
    ctx.accounts.order.state = OrderState::Closed.into();

    // TODO Transfer funds out
    emit_cpi!(Order::get_edit_event(
        &mut ctx.accounts.order.clone(),
        ctx.accounts.order.key(),
        ctx.accounts.market.market_identifier,
        OrderEditType::Close,
    ));
    Ok(())
}
