use anchor_lang::{prelude::*, solana_program::sysvar};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use mpl_token_metadata::accounts::Metadata;
use program_utils::{get_bump_in_seed_form, pnft::utils::get_is_pnft, ExtraTransferParams};

use crate::{
    errors::MarketError,
    state::*,
    utils::{
        get_fee_amount, parse_remaining_accounts, pay_royalties, transfer_nft, transfer_sol,
        unfreeze_nft,
    },
};

#[derive(Accounts)]
#[instruction()]
#[event_cpi]
pub struct FillSellOrder<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    #[account(
        mut,
        constraint = order.owner == seller.key(),
    )]
    /// CHECK: constraint check
    pub seller: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [WALLET_SEED,
        seller.key().as_ref()],
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
        mut,
        constraint = Order::is_active(order.state),
        constraint = order.market == market.key(),
        seeds = [ORDER_SEED,
        order.nonce.as_ref(),
        order.market.as_ref(),
        order.owner.as_ref()],
        bump,
        close = seller
    )]
    pub order: Box<Account<'info, Order>>,
    #[account(mut)]
    pub nft_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    /// CHECK: deser. in Account
    pub nft_metadata: UncheckedAccount<'info>,
    /// CHECK: constraint check in multiple CPI calls
    pub nft_edition: UncheckedAccount<'info>,
    #[account(
        mut,
        associated_token::mint = nft_mint,
        associated_token::authority = seller,
    )]
    pub seller_nft_ta: Box<Account<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = initializer,
        associated_token::mint = nft_mint,
        associated_token::authority = initializer,
    )]
    pub buyer_nft_ta: Box<Account<'info, TokenAccount>>,
    /// CHECK: constraint
    #[account(
        mut,
        constraint = treasury.key().to_string() == PROTOCOL_TREASURY
    )]
    pub treasury: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    /// CHECK: checked by constraint and in cpi
    #[account(address = sysvar::instructions::id())]
    pub sysvar_instructions: UncheckedAccount<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// CHECK: checked by constraint and in cpi
    pub token_metadata_program: UncheckedAccount<'info>,
}

//remaining accounts
// 0 token_record or default,
// 1 authorization_rules or default,
// 2 authorization_rules_program or default,
// 4 delegate record or default,
// 5 buyer token record or default,
// 6-13 optional creator accounts in order of metadata. Will error if is pnft and correct creator accounts are not present

/// Initializer is the buyer and is buying an nft from the seller
/// The seller is the owner of the order account
/// Buyer transfers sol to seller account
#[inline(always)]
pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, FillSellOrder<'info>>) -> Result<()> {
    let bump = &get_bump_in_seed_form(&ctx.bumps.wallet);

    let parsed_accounts = parse_remaining_accounts(
        ctx.remaining_accounts.to_vec(),
        ctx.accounts.initializer.key(),
        ctx.accounts.order.fees_on,
        false,
        Some(1),
    );

    let dest_token_record =
        if ctx.remaining_accounts.get(4).cloned().unwrap().key() == Pubkey::default() {
            None
        } else {
            ctx.remaining_accounts.get(4).cloned()
        };

    let pnft_params = parsed_accounts.pnft_params;

    let signer_seeds = &[&[WALLET_SEED, ctx.accounts.order.owner.as_ref(), bump][..]];

    let nft_authority = ctx.accounts.wallet.to_account_info();
    let sol_holder = ctx.accounts.initializer.to_account_info();

    // validate seller
    if ctx.accounts.order.owner != ctx.accounts.seller.key() {
        return Err(MarketError::WrongAccount.into());
    }

    if parsed_accounts.fees_on {
        let fee_amount = get_fee_amount(ctx.accounts.order.price);
        transfer_sol(
            sol_holder.clone(),
            ctx.accounts.treasury.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            Some(signer_seeds),
            fee_amount,
        )?;
        // transfer sol from buyer to seller
        transfer_sol(
            sol_holder,
            ctx.accounts.seller.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            Some(signer_seeds),
            ctx.accounts.order.price,
        )?;
    } else {
        // transfer sol from buyer to seller
        transfer_sol(
            sol_holder,
            ctx.accounts.seller.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            Some(signer_seeds),
            ctx.accounts.order.price,
        )?;
    }

    let metadata =
        Metadata::safe_deserialize(&ctx.accounts.nft_metadata.to_account_info().data.borrow())?;
    let is_pnft = get_is_pnft(&metadata);

    // unfreeze nft first so that a transfer can be made
    if !is_pnft {
        unfreeze_nft(
            ctx.accounts.seller.to_account_info(),
            ctx.accounts.initializer.to_account_info(),
            ctx.accounts.nft_mint.to_account_info(),
            ctx.accounts.seller_nft_ta.to_account_info(),
            ctx.accounts.wallet.to_account_info(),
            ctx.accounts.nft_metadata.to_account_info(),
            ctx.accounts.nft_edition.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.sysvar_instructions.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.associated_token_program.to_account_info(),
            ctx.accounts.token_metadata_program.to_account_info(),
            signer_seeds,
            pnft_params.clone(),
        )?;
    }

    // transfer nft
    transfer_nft(
        nft_authority,
        ctx.accounts.seller.to_account_info(),
        ctx.accounts.initializer.to_account_info(),
        ctx.accounts.initializer.to_account_info(),
        ctx.accounts.nft_mint.to_account_info(),
        ctx.accounts.nft_metadata.to_account_info(),
        ctx.accounts.nft_edition.to_account_info(),
        ctx.accounts.seller_nft_ta.to_account_info(),
        ctx.accounts.buyer_nft_ta.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.sysvar_instructions.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.associated_token_program.to_account_info(),
        ctx.accounts.token_metadata_program.to_account_info(),
        ExtraTransferParams {
            dest_token_record,
            owner_token_record: pnft_params.token_record.clone(),
            authorization_rules: pnft_params.authorization_rules.clone(),
            authorization_rules_program: pnft_params.authorization_rules_program.clone(),
            authorization_data: None,
        },
        signer_seeds,
    )?;

    if is_pnft {
        pay_royalties(
            ctx.accounts.order.price,
            ctx.accounts.nft_metadata.to_account_info().clone(),
            ctx.accounts.initializer.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            parsed_accounts.creator_accounts,
            false,
            Some(signer_seeds),
        )?;
    }

    // close order account
    msg!("Close sell order account: {}", ctx.accounts.order.key());
    ctx.accounts.order.state = OrderState::Closed.into();
    emit_cpi!(Order::get_edit_event(
        &mut ctx.accounts.order.clone(),
        ctx.accounts.order.key(),
        ctx.accounts.market.market_identifier,
        OrderEditType::FillAndClose,
    ));

    Ok(())
}
