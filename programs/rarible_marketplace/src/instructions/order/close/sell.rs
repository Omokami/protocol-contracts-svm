use anchor_lang::prelude::*;
use anchor_lang::{solana_program::sysvar, Key};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use mpl_token_metadata::accounts::Metadata;
use mpl_token_metadata::types::RevokeArgs;

use crate::utils::metaplex::pnft::utils::get_is_pnft;
use crate::utils::{get_bump_in_seed_form, parse_remaining_accounts_pnft, revoke_nft, thaw_nft, ExtraRevokeParams, FreezeNft, RevokeNft};
use crate::state::*;

#[derive(Accounts)]
#[instruction()]
#[event_cpi]
pub struct CloseSellOrder<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    #[account(
        mut,
        constraint = order.owner == initializer.key(),
        constraint = order.market == market.key(),
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
        constraint = Market::is_active(market.state),
        seeds = [MARKET_SEED,
        market.market_identifier.as_ref()],
        bump,
    )]
    pub market: Box<Account<'info, Market>>,
    #[account(
        mut,
        seeds = [WALLET_SEED,
        order.owner.as_ref()],
        bump,
    )]
    pub wallet: Box<Account<'info, Wallet>>,
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
        associated_token::authority = initializer,
    )]
    pub nft_ta: Box<Account<'info, TokenAccount>>,
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
// 4 delegate record or default

#[inline(always)]
pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, CloseSellOrder<'info>>) -> Result<()> {
    msg!("Close sell order account: {}", ctx.accounts.order.key());

    let parsed_remaining_accounts = parse_remaining_accounts_pnft(
        ctx.remaining_accounts.to_vec(),
        false,
        None,
    );

    let pnft_params = parsed_remaining_accounts.pnft_params;

    let bump = &get_bump_in_seed_form(&ctx.bumps.wallet);

    let signer_seeds = &[&[WALLET_SEED, ctx.accounts.order.owner.as_ref(), bump][..]];

    let metadata =
        Metadata::safe_deserialize(&ctx.accounts.nft_metadata.to_account_info().data.borrow())?;

    let is_pnft = get_is_pnft(&metadata);

    // unfreeze nft if not pnft
    if !is_pnft {
        let thaw_accounts = FreezeNft {
            authority: ctx.accounts.initializer.to_account_info(),
            payer: ctx.accounts.initializer.to_account_info(),
            token_owner: ctx.accounts.initializer.to_account_info(),
            token: ctx.accounts.nft_ta.to_account_info(),
            delegate: ctx.accounts.initializer.to_account_info(),
            mint: ctx.accounts.nft_mint.to_account_info(),
            metadata: ctx.accounts.nft_metadata.to_account_info(),
            edition: ctx.accounts.nft_edition.to_account_info(),
            mpl_token_metadata: ctx.accounts.token_metadata_program.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            instructions: ctx.accounts.sysvar_instructions.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            ata_program: ctx.accounts.associated_token_program.to_account_info(),
        };
        let thaw_ctx = CpiContext::new_with_signer(ctx.accounts.token_metadata_program.to_account_info(), thaw_accounts, signer_seeds);
        thaw_nft(thaw_ctx, pnft_params)?;
    } else {
        let revoke_accounts = RevokeNft {
            token: ctx.accounts.nft_ta.to_account_info(),
            delegate: ctx.accounts.initializer.to_account_info(),
            metadata: ctx.accounts.nft_metadata.to_account_info(),
            mint: ctx.accounts.nft_mint.to_account_info(),
            authority: ctx.accounts.initializer.to_account_info(),
            payer: ctx.accounts.initializer.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            sysvar_instructions: ctx.accounts.sysvar_instructions.to_account_info(),
        };

        let revoke_params = ExtraRevokeParams {
            delegate_record: None,
            master_edition: None,
            token_record: pnft_params.token_record,
            authorization_rules_program: pnft_params.authorization_rules_program,
            authorization_rules: pnft_params.authorization_rules,
            revoke_args: RevokeArgs::SaleV1,
        };

        let revoke_ctx = CpiContext::new_with_signer(ctx.accounts.token_metadata_program.to_account_info(), revoke_accounts, signer_seeds);
        
        //revoke nft if pnft
        revoke_nft(revoke_ctx, revoke_params)?;
    }

    ctx.accounts.order.state = OrderState::Closed.into();

    emit_cpi!(Order::get_edit_event(
        &mut ctx.accounts.order.clone(),
        ctx.accounts.order.key(),
        ctx.accounts.market.market_identifier,
        OrderEditType::Close,
    ));
    Ok(())
}
