use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{revoke, Mint, Revoke, TokenAccount, TokenInterface},
};

use crate::{state::*, utils::get_bump_in_seed_form};

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
    #[account(
        mut,
        associated_token::mint = payment_mint,
        associated_token::authority = initializer,
        associated_token::token_program = payment_token_program,
    )]
    pub initializer_payment_ta: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut, constraint = payment_mint.key() == order.payment_mint)]
    pub payment_mint: Box<InterfaceAccount<'info, Mint>>,
    pub payment_token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> CancelBid<'info> {
    fn revoke_payment(&self, signer_seeds: &[&[&[u8]]]) -> Result<()> {
        let cpi_ctx = CpiContext::new_with_signer(
            self.payment_token_program.to_account_info(),
            Revoke {
                authority: self.order.to_account_info(),
                source: self.initializer_payment_ta.to_account_info(),
            },
            signer_seeds,
        );
        revoke(cpi_ctx)
    }
}

#[inline(always)]
pub fn handler(ctx: Context<CancelBid>) -> Result<()> {
    msg!("Close buy order account: {}", ctx.accounts.order.key());
    ctx.accounts.order.state = OrderState::Closed.into();
    let bump = &get_bump_in_seed_form(&ctx.bumps.order);

    let signer_seeds: &[&[&[u8]]; 1] = &[&[
        ORDER_SEED,
        ctx.accounts.order.nonce.as_ref(),
        ctx.accounts.order.market.as_ref(),
        ctx.accounts.order.owner.as_ref(),
        bump,
    ][..]];

    // TODO Transfer funds out
    ctx.accounts.revoke_payment(signer_seeds)?;
    emit_cpi!(Order::get_edit_event(
        &mut ctx.accounts.order.clone(),
        ctx.accounts.order.key(),
        ctx.accounts.market.market_identifier,
        OrderEditType::Close,
    ));
    Ok(())
}
