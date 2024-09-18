use anchor_lang::prelude::*;

use crate::{state::*, utils::parse_remaining_accounts};

use super::InitOrderData;

#[derive(Accounts)]
#[instruction(data: InitOrderData)]
#[event_cpi]
pub struct InitBuyOrder<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    #[account(
        mut,
        // make sure bidding wallet has enough balance to place the order
        constraint = wallet.balance >= data.price.checked_mul(data.size).unwrap(),
        seeds = [WALLET_SEED,
        initializer.key().as_ref()],
        bump,
    )]
    pub wallet: Box<Account<'info, Wallet>>,
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

//remaining accounts
// 0 token_record or default,
// 1 authorization_rules or default,
// 2 authorization_rules_program or default

#[inline(always)]
pub fn handler(ctx: Context<InitBuyOrder>, data: InitOrderData) -> Result<()> {
    msg!("Initialize a new buy order: {}", ctx.accounts.order.key());

    let parsed_accounts = parse_remaining_accounts(
        ctx.remaining_accounts.to_vec(),
        ctx.accounts.initializer.key(),
        true,
        false,
        None,
    );

    let clock = Clock::get()?;
    // create a new order with size 1
    Order::init(
        &mut ctx.accounts.order,
        ctx.accounts.market.key(),
        ctx.accounts.initializer.key(),
        ctx.accounts.wallet.key(),
        data.nonce,
        ctx.accounts.nft_mint.key(),
        clock.unix_timestamp,
        OrderSide::Buy.into(),
        data.size,
        data.price,
        OrderState::Ready.into(),
        parsed_accounts.fees_on,
    );

    emit_cpi!(Order::get_edit_event(
        &mut ctx.accounts.order.clone(),
        ctx.accounts.order.key(),
        ctx.accounts.market.market_identifier,
        OrderEditType::Init,
    ));

    Ok(())
}
