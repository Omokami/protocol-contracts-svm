use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{approve, Approve, Mint, TokenAccount, TokenInterface},
};

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
    #[account(
        mut,
        associated_token::mint = payment_mint,
        associated_token::authority = initializer,
        associated_token::token_program = payment_token_program,
    )]
    pub initializer_payment_ta: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub payment_mint: Box<InterfaceAccount<'info, Mint>>,
    pub payment_token_program: Interface<'info, TokenInterface>,
    /// CHECK: can be anything
    pub nft_mint: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> BidNft<'info> {
    fn delegate_payment(&self, amount: u64) -> Result<()> {
        let cpi_ctx = CpiContext::new(
            self.payment_token_program.to_account_info(),
            Approve {
                to: self.initializer_payment_ta.to_account_info(),
                authority: self.initializer.to_account_info(),
                delegate: self.order.to_account_info(),
            },
        );
        approve(cpi_ctx, amount)
    }
}

#[inline(always)]
pub fn handler(ctx: Context<BidNft>, data: BidData) -> Result<()> {
    msg!("Initialize a new buy order: {}", ctx.accounts.order.key());

    let clock = Clock::get()?;
    let bid_value = data.size.checked_mul(data.price).unwrap();
    // Transfer bid funds TODO;

    ctx.accounts.delegate_payment(bid_value)?;
    // create a new order with size 1
    Order::init(
        &mut ctx.accounts.order,
        ctx.accounts.market.key(),
        ctx.accounts.initializer.key(),
        data.nonce,
        ctx.accounts.nft_mint.key(),
        ctx.accounts.payment_mint.key(),
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
