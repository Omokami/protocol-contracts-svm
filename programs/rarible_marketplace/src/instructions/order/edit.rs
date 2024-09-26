use anchor_lang::prelude::*;

use crate::state::*;

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy, PartialEq)]
pub struct EditOrderData {
    pub new_price: u64,
    pub new_size: u64,
}

#[derive(Accounts)]
#[instruction(data: EditOrderData)]
#[event_cpi]
pub struct EditOrder<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    #[account(
        constraint = market.key() == order.market,
        seeds = [MARKET_SEED,
        market.market_identifier.as_ref()],
        bump,
    )]
    pub market: Box<Account<'info, Market>>,
    #[account(
        mut,
        constraint = order.owner == initializer.key(),
        constraint = data.new_size > 0 && data.new_price > 0,
        constraint = Order::is_active(order.state),
        seeds = [ORDER_SEED,
        order.nonce.as_ref(),
        order.market.key().as_ref(),
        initializer.key().as_ref()],
        bump,
    )]
    pub order: Box<Account<'info, Order>>,
    pub system_program: Program<'info, System>,
}

#[inline(always)]
pub fn handler(ctx: Context<EditOrder>, data: EditOrderData) -> Result<()> {
    msg!("Edit buy order: {}", ctx.accounts.order.key());
    let clock = Clock::get()?;

    let order_side: u8 = ctx.accounts.order.side;

    if order_side == 1 {
        // edit the order without size
        Order::edit_sell(
            &mut ctx.accounts.order,
            data.new_price,
            clock.unix_timestamp,
        );
    } else if order_side == 0 {
        // edit the order with size
        Order::edit_buy(
            &mut ctx.accounts.order,
            data.new_price,
            data.new_size,
            clock.unix_timestamp,
        );
    } else {
        // error, shouldnt happen
    }

    emit_cpi!(Order::get_edit_event(
        &mut ctx.accounts.order.clone(),
        ctx.accounts.order.key(),
        ctx.accounts.market.market_identifier,
        OrderEditType::Edit,
    ));

    Ok(())
}
