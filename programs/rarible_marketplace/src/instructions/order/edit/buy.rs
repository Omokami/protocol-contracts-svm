use anchor_lang::prelude::*;

use crate::state::*;

use super::EditBuyOrderData;

#[derive(Accounts)]
#[instruction(data: EditBuyOrderData)]
#[event_cpi]
pub struct EditBuyOrder<'info> {
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
    #[account(
        mut,
        // make sure bidding wallet has enough balance to place the order
        constraint = wallet.balance >= data.new_price.checked_mul(data.new_size).unwrap(),
        seeds = [WALLET_SEED,
        initializer.key().as_ref()],
        bump,
    )]
    pub wallet: Box<Account<'info, Wallet>>,
    pub system_program: Program<'info, System>,
}

#[inline(always)]
pub fn handler(ctx: Context<EditBuyOrder>, data: EditBuyOrderData) -> Result<()> {
    msg!("Edit buy order: {}", ctx.accounts.order.key());
    let clock = Clock::get()?;
    // edit the order with size
    Order::edit_buy(
        &mut ctx.accounts.order,
        data.new_price,
        data.new_size,
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
