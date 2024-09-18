use anchor_lang::prelude::*;

use crate::state::*;

#[derive(Accounts)]
#[instruction()]
#[event_cpi]
pub struct InitMarket<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    #[account()]
    /// CHECK: doesn't actually need to be a mint
    pub market_identifier: UncheckedAccount<'info>,
    #[account(
        init,
        seeds = [MARKET_SEED,
        market_identifier.key().as_ref()],
        bump,
        payer = initializer,
        space = 8 + std::mem::size_of::<Market>()
    )]
    pub market: Box<Account<'info, Market>>,
    pub system_program: Program<'info, System>,
}

#[inline(always)]
pub fn handler(ctx: Context<InitMarket>) -> Result<()> {
    msg!("Initializing new market");
    Market::init(
        &mut ctx.accounts.market,
        ctx.accounts.market_identifier.key(),
        ctx.accounts.initializer.key(),
    );

    emit_cpi!(Market::get_edit_event(
        &mut ctx.accounts.market.clone(),
        ctx.accounts.market.key(),
        MarketEditType::Init,
    ));
    Ok(())
}
