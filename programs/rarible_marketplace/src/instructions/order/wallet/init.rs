use anchor_lang::prelude::*;
use program_utils::get_bump_in_seed_form;

use crate::{state::*, utils::transfer_sol};

#[derive(Accounts)]
#[instruction()]
#[event_cpi]
pub struct InitBiddingWallet<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    #[account(
        init,
        seeds = [WALLET_SEED,
        initializer.key().as_ref()],
        payer = initializer,
        space = 8 + std::mem::size_of::<Wallet>(),
        bump,
    )]
    pub wallet: Box<Account<'info, Wallet>>,
    pub system_program: Program<'info, System>,
}

#[inline(always)]
pub fn handler(ctx: Context<InitBiddingWallet>, amount: u64) -> Result<()> {
    msg!("Initializing a new wallet: {}", ctx.accounts.wallet.key());

    let bump = &get_bump_in_seed_form(&ctx.bumps.wallet);

    let signer_seeds = &[&[WALLET_SEED, ctx.accounts.initializer.key.as_ref(), bump][..]];

    // transfer the amount to the wallet account to initializer if amount > 0
    if amount > 0 {
        transfer_sol(
            ctx.accounts.initializer.to_account_info(),
            ctx.accounts.wallet.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            Some(signer_seeds),
            amount,
        )?;
    }

    Wallet::init(
        &mut ctx.accounts.wallet,
        ctx.accounts.initializer.key(),
        amount,
    );

    emit_cpi!(Wallet::get_edit_event(
        &mut ctx.accounts.wallet.clone(),
        ctx.accounts.wallet.key(),
        WalletEditType::Init,
    ));
    Ok(())
}
