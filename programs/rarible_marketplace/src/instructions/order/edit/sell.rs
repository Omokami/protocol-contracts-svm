use crate::state::*;
use anchor_lang::prelude::*;

use super::EditSellOrderData;

// sell order can only have its price edited

#[derive(Accounts)]
#[instruction(data: EditSellOrderData)]
#[event_cpi]
pub struct EditSellOrder<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    #[account(
        mut,
        constraint = order.owner == initializer.key(),
        constraint = data.new_price > 0,
        constraint = Order::is_active(order.state),
        seeds = [ORDER_SEED,
        order.nonce.as_ref(),
        order.market.key().as_ref(),
        order.owner.as_ref()],
        bump,
    )]
    pub order: Box<Account<'info, Order>>,
    #[account(
        constraint = Market::is_active(market.state),
        constraint = market.key() == order.market,
        seeds = [MARKET_SEED,
        market.market_identifier.as_ref()],
        bump,
    )]
    pub market: Box<Account<'info, Market>>,
}

#[inline(always)]
pub fn handler(ctx: Context<EditSellOrder>, data: EditSellOrderData) -> Result<()> {
    msg!("Edit sell order: {}", ctx.accounts.order.key());
    let clock = Clock::get()?;
    // update the sell order account
    Order::edit_sell(
        &mut ctx.accounts.order,
        data.new_price,
        clock.unix_timestamp,
    );

    emit_cpi!(Order::get_edit_event(
        &mut ctx.accounts.order.clone(),
        ctx.accounts.order.key(),
        ctx.accounts.market.market_identifier,
        OrderEditType::Edit,
    ));
    Ok(())
}
