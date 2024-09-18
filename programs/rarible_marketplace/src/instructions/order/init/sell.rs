use anchor_lang::{prelude::*, solana_program::sysvar};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use mpl_token_metadata::{accounts::Metadata, types::DelegateArgs};
use program_utils::{get_bump_in_seed_form, pnft::utils::get_is_pnft, ExtraDelegateParams};
use vault::state::{Appraisal, APPRAISAL_SEED};

use crate::{
    state::*,
    utils::{delegate_nft, freeze_nft, parse_remaining_accounts},
};

use super::InitOrderData;

#[derive(Accounts)]
#[instruction(data: InitOrderData)]
#[event_cpi]
pub struct InitSellOrder<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    #[account(
        mut,
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
    #[account(
        seeds = [APPRAISAL_SEED, market.market_identifier.as_ref(), nft_mint.key().as_ref()],
        bump,
        seeds::program = vault::ID,
    )]
    pub appraisal: Box<Account<'info, Appraisal>>,
    #[account(mut)]
    pub nft_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    /// CHECK: deser. in Account
    pub nft_metadata: UncheckedAccount<'info>,
    /// CHECK: checked in cpi
    #[account(mut)]
    pub nft_edition: UncheckedAccount<'info>,
    #[account(
        mut,
        constraint = nft_ta.owner == initializer.key(),
        constraint = nft_ta.mint == nft_mint.key(),
    )]
    pub nft_ta: Box<Account<'info, TokenAccount>>,
    /// CHECK: checked by constraint and in cpi
    #[account(address = sysvar::instructions::id())]
    pub sysvar_instructions: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// CHECK: checked by constraint and in cpi
    pub token_metadata_program: UncheckedAccount<'info>,
}
//remaining accounts
// 0 token_record or default,
// 1 authorization_rules or default,
// 2 authorization_rules_program or default,
// 4 delegate record or default,
// 5 existing delegate or default,
// 6 existing delegate record or default

#[inline(always)]
pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, InitSellOrder<'info>>,
    data: InitOrderData,
) -> Result<()> {
    msg!("Initialize a new sell order: {}", ctx.accounts.order.key());

    let parsed_accounts = parse_remaining_accounts(
        ctx.remaining_accounts.to_vec(),
        ctx.accounts.initializer.key(),
        true,
        true,
        None,
    );

    let pnft_params = parsed_accounts.pnft_params;

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
        OrderSide::Sell.into(),
        1, // always 1
        data.price,
        OrderState::Ready.into(),
        parsed_accounts.fees_on,
    );

    let bump = &get_bump_in_seed_form(&ctx.bumps.wallet);

    let signer_seeds = &[&[WALLET_SEED, ctx.accounts.initializer.key.as_ref(), bump][..]];

    let metadata = Metadata::safe_deserialize(&ctx.accounts.nft_metadata.data.borrow())?;

    let is_pnft = get_is_pnft(&metadata);

    // freeze the nft of the seller with the bidding wallet account as the authority
    delegate_nft(
        ctx.accounts.initializer.to_account_info(),
        ctx.accounts.initializer.to_account_info(),
        ctx.accounts.nft_mint.to_account_info(),
        ctx.accounts.nft_ta.to_account_info(),
        ctx.accounts.nft_metadata.to_account_info(),
        ctx.accounts.wallet.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.sysvar_instructions.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.token_metadata_program.to_account_info(),
        signer_seeds,
        ExtraDelegateParams {
            master_edition: Some(ctx.accounts.nft_edition.to_account_info()),
            delegate_record: parsed_accounts.delegate_record.clone(),
            token_record: pnft_params.token_record.clone(),
            authorization_rules_program: pnft_params.authorization_rules_program.clone(),
            authorization_rules: pnft_params.authorization_rules.clone(),
            token: Some(ctx.accounts.nft_ta.to_account_info()),
            spl_token_program: Some(ctx.accounts.token_program.to_account_info()),
            delegate_args: DelegateArgs::SaleV1 {
                amount: 1,
                authorization_data: None,
            },
            existing_delegate_params: parsed_accounts.existing_delegate_params,
        },
    )?;
    if !is_pnft {
        freeze_nft(
            ctx.accounts.initializer.to_account_info(),
            ctx.accounts.initializer.to_account_info(),
            ctx.accounts.nft_mint.to_account_info(),
            ctx.accounts.nft_ta.to_account_info(),
            ctx.accounts.nft_metadata.to_account_info(),
            ctx.accounts.nft_edition.to_account_info(),
            ctx.accounts.wallet.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.sysvar_instructions.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.associated_token_program.to_account_info(),
            ctx.accounts.token_metadata_program.to_account_info(),
            signer_seeds,
            pnft_params,
        )?;
    }

    emit_cpi!(Order::get_edit_event(
        &mut ctx.accounts.order.clone(),
        ctx.accounts.order.key(),
        ctx.accounts.market.market_identifier,
        OrderEditType::Init,
    ));

    Ok(())
}
