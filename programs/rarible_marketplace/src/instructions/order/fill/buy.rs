use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use mpl_token_metadata::accounts::Metadata;
use program_utils::{lamport_transfer, pnft::utils::get_is_pnft, ExtraTransferParams};

use crate::{
    state::*,
    utils::{parse_remaining_accounts, pay_royalties, transfer_nft},
};

#[derive(Accounts)]
#[instruction()]
#[event_cpi]
pub struct FillBuyOrder<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    #[account(
        mut,
        constraint = order.owner == buyer.key(),
    )]
    /// CHECK: constraint check
    pub buyer: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [WALLET_SEED,
        order.owner.as_ref()],
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
    )]
    pub order: Box<Account<'info, Order>>,
    #[account(
        mut,
        constraint = order.nft_mint == Pubkey::default() || order.nft_mint == nft_mint.key()
    )]
    pub nft_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    /// CHECK: deser. in Account
    pub nft_metadata: UncheckedAccount<'info>,
    /// CHECK: constraint check in multiple CPI calls
    pub nft_edition: UncheckedAccount<'info>,
    #[account(
        mut,
        associated_token::mint = nft_mint,
        associated_token::authority = initializer,
    )]
    pub seller_nft_ta: Box<Account<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = initializer,
        associated_token::mint = nft_mint,
        associated_token::authority = buyer,
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
    pub sysvar_instructions: UncheckedAccount<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// CHECK: checked by constraint and in cpi
    pub token_metadata_program: UncheckedAccount<'info>,
}

//remaining accounts
// 0 token_record or default,
// 1 authorization_rules or default,
// 2 authorization_rules_program or default,
//
// 4 delegate record or default,
// 5 seller token record or default,
// 6-11 optional creator accounts in order of metadata. Will error if is pnft and correct creator accounts are not present

/// seller is initializer and is transferring the nft to buyer who is the owner of the order account
/// buyer is the owner of the order account and is transferring sol to seller via bidding wallet
#[inline(always)]
pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, FillBuyOrder<'info>>) -> Result<()> {
    let parsed_accounts = parse_remaining_accounts(
        ctx.remaining_accounts.to_vec(),
        ctx.accounts.initializer.key(),
        ctx.accounts.order.fees_on,
        false,
        Some(1),
    );

    let pnft_params = parsed_accounts.pnft_params;

    // edit wallet account to decrease balance
    msg!("Edit wallet balance: {}", ctx.accounts.wallet.key());
    Wallet::edit_balance(&mut ctx.accounts.wallet, false, ctx.accounts.order.price);

    let buyer_token_record =
        if ctx.remaining_accounts.get(4).cloned().unwrap().key() == Pubkey::default() {
            None
        } else {
            ctx.remaining_accounts.get(4).cloned()
        };

    // transfer nft
    transfer_nft(
        ctx.accounts.initializer.to_account_info(),
        ctx.accounts.initializer.to_account_info(),
        ctx.accounts.initializer.to_account_info(),
        ctx.accounts.buyer.to_account_info(),
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
            owner_token_record: pnft_params.token_record,
            dest_token_record: buyer_token_record,
            authorization_rules: pnft_params.authorization_rules,
            authorization_rules_program: pnft_params.authorization_rules_program.clone(),
            authorization_data: None,
        },
        &[],
    )?;

    lamport_transfer(
        ctx.accounts.wallet.to_account_info(),
        ctx.accounts.initializer.to_account_info(),
        ctx.accounts.order.price,
    )?;

    // edit order
    let price = ctx.accounts.order.price;
    let size = ctx.accounts.order.size;

    let clock = Clock::get()?;
    Order::edit_buy(
        &mut ctx.accounts.order,
        price,
        size - 1,
        clock.unix_timestamp,
    );

    let metadata =
        Metadata::safe_deserialize(&ctx.accounts.nft_metadata.to_account_info().data.borrow())?;

    if get_is_pnft(&metadata) {
        pay_royalties(
            ctx.accounts.order.price,
            ctx.accounts.nft_metadata.to_account_info().clone(),
            ctx.accounts.initializer.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            parsed_accounts.creator_accounts,
            true,
            None,
        )?;
    }

    if size == 1 {
        // close order account
        msg!(
            "Close buy order account: {}: {}",
            ctx.accounts.order.key(),
            ctx.accounts.market.market_identifier
        );
        ctx.accounts.order.state = OrderState::Closed.into();
        emit_cpi!(Order::get_edit_event(
            &mut ctx.accounts.order.clone(),
            ctx.accounts.order.key(),
            ctx.accounts.market.market_identifier,
            OrderEditType::FillAndClose,
        ));
        ctx.accounts
            .order
            .close(ctx.accounts.buyer.to_account_info())?;
    } else {
        emit_cpi!(Order::get_edit_event(
            &mut ctx.accounts.order.clone(),
            ctx.accounts.order.key(),
            ctx.accounts.market.market_identifier,
            OrderEditType::Fill,
        ));
        msg!("Filled buy order: {}", ctx.accounts.order.key());
    }

    emit_cpi!(Wallet::get_edit_event(
        &mut ctx.accounts.wallet.clone(),
        ctx.accounts.wallet.key(),
        WalletEditType::Edit,
    ));

    Ok(())
}
