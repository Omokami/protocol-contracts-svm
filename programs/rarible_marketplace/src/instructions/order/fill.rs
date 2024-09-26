use anchor_lang::{prelude::*, solana_program::sysvar};
use anchor_spl::{
    associated_token::AssociatedToken, token_2022::TransferChecked as Token22TransferChecked, token_interface::{Mint, TokenAccount, TokenInterface}
};
use mpl_token_metadata::accounts::Metadata;
use wen_new_standard::cpi::{accounts::ApproveTransfer, approve_transfer};

use crate::{
    errors::MarketError,
    state::*,
    utils::{
        get_bump_in_seed_form, get_fee_amount, metaplex::pnft::utils::get_is_pnft, mplx_transfer::{transfer_metaplex_nft, ExtraTransferParams, MetaplexAdditionalTransferAccounts, TransferMetaplexNft}, parse_remaining_accounts_pnft, pay_royalties, token_extensions::{transfer_token22_checked, WnsApprovalAccounts}
    },
};

#[derive(Accounts)]
#[instruction()]
#[event_cpi]
pub struct FillOrder<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,
    #[account(mut)]
    /// CHECK: constraint check
    pub maker: UncheckedAccount<'info>,
    /// CHECK: constraint check
    pub buyer: UncheckedAccount<'info>,
    /// CHECK: constraint check
    pub seller: UncheckedAccount<'info>,
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
        constraint = order.owner == maker.key(),
        seeds = [ORDER_SEED,
        order.nonce.as_ref(),
        order.market.as_ref(),
        order.owner.as_ref()],
        bump,
        close = maker
    )]
    pub order: Box<Account<'info, Order>>,
    #[account(mut)]
    pub nft_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        associated_token::mint = nft_mint,
        associated_token::authority = seller,
        associated_token::token_program = nft_token_program,
    )]
    pub seller_nft_ta: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = nft_mint,
        associated_token::authority = buyer,
        associated_token::token_program = nft_token_program,
    )]
    pub buyer_nft_ta: Box<InterfaceAccount<'info, TokenAccount>>,
    pub system_program: Program<'info, System>,
    pub nft_token_program: Interface<'info, TokenInterface>,
    /// CHECK: checked by constraint and in cpi
    pub nft_program: UncheckedAccount<'info>,
    /// CHECK: checked by constraint and in cpi
    #[account(address = sysvar::instructions::id())]
    pub sysvar_instructions: UncheckedAccount<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// CHECK: checked by constraint and in cpi
    pub token_metadata_program: UncheckedAccount<'info>,
}

impl<'info> FillOrder<'info> {

    /*
        Metaplex Transfer Instructions
    */ 

    fn metaplex_nft_transfer(&self, is_buy: bool, is_pnft: bool, extra_transfer_accounts: MetaplexAdditionalTransferAccounts<'info>) -> Result<()> {
        let transfer_accounts = if is_buy {
            TransferMetaplexNft {
                authority: self.taker.to_account_info(),
                payer: self.taker.to_account_info(),
                source_owner: self.taker.to_account_info(),
                source_ta: self.seller_nft_ta.to_account_info(),
                destination_owner: self.maker.to_account_info(),
                destination_ta: self.buyer_nft_ta.to_account_info(),
                mint: self.nft_mint.to_account_info(),
                metadata: extra_transfer_accounts.metadata.to_account_info(),
                edition: extra_transfer_accounts.edition.to_account_info(),
                system_program: self.system_program.to_account_info(),
                instructions: self.sysvar_instructions.to_account_info(),
                token_program: self.nft_token_program.to_account_info(),
                ata_program: self.associated_token_program.to_account_info(),
            }
        } else {
            TransferMetaplexNft {
                authority: self.order.to_account_info(),
                payer: self.taker.to_account_info(),
                source_owner: self.order.to_account_info(),
                source_ta: self.seller_nft_ta.to_account_info(),
                destination_owner: self.taker.to_account_info(),
                destination_ta: self.buyer_nft_ta.to_account_info(),
                mint: self.nft_mint.to_account_info(),
                metadata: extra_transfer_accounts.metadata.to_account_info(),
                edition: extra_transfer_accounts.edition.to_account_info(),
                system_program: self.system_program.to_account_info(),
                instructions: self.sysvar_instructions.to_account_info(),
                token_program: self.nft_token_program.to_account_info(),
                ata_program: self.associated_token_program.to_account_info(),
            }
        };
        
        let cpi_program = self.nft_program.to_account_info();

        let transfer_ctx = CpiContext::new(cpi_program, transfer_accounts);
        // transfer nft
        transfer_metaplex_nft(
            transfer_ctx,
            extra_transfer_accounts.extra_accounts,
            1,
            is_pnft
        )
    }

    /*
        Compressed Transfer Instructions
    */

    /*
        Token 22 Transfer Instructions
    */

    // WNS Pre-Transfer Approval
    fn approve_wns_transfer(&self, buy_amount: u64, wns_accounts: WnsApprovalAccounts<'info>) -> Result<()> {
        let cpi_program = self.nft_program.to_account_info();
        let cpi_accounts = ApproveTransfer {
            payer: self.taker.to_account_info(),
            authority: self.taker.to_account_info(),
            mint: self.nft_mint.to_account_info(),
            approve_account: wns_accounts.approval_account.to_account_info(),
            payment_mint: self.system_program.to_account_info(), // TODO
            distribution_token_account: None, // TODO
            authority_token_account: None, // TODO
            distribution_account: wns_accounts.distribution_account.to_account_info(),
            system_program: self.system_program.to_account_info(),
            distribution_program: wns_accounts.distribution_program.to_account_info(),
            token_program: self.nft_token_program.to_account_info(),
            payment_token_program: None, // TODO
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        approve_transfer(cpi_ctx, buy_amount)
    }

    // General Token22 Transfer
    fn token22_nft_transfer(&self, is_buy: bool, remaining_accounts: Vec<AccountInfo<'info>>) -> Result<()> {
        let transfer_cpi = if is_buy {
            CpiContext::new(
                self.nft_token_program.to_account_info(),
                Token22TransferChecked {
                    from: self.seller_nft_ta.to_account_info(),
                    to: self.buyer_nft_ta.to_account_info(),
                    authority: self.order.to_account_info(),
                    mint: self.nft_mint.to_account_info(),
                },
            )
        } else {
            CpiContext::new(
                self.nft_token_program.to_account_info(),
                Token22TransferChecked {
                    from: self.seller_nft_ta.to_account_info(),
                    to: self.buyer_nft_ta.to_account_info(),
                    authority: self.taker.to_account_info(),
                    mint: self.nft_mint.to_account_info(),
                },
            )
        };

        transfer_token22_checked(
            transfer_cpi.with_remaining_accounts(remaining_accounts),
            1, // supply = 1
            0, // decimals = 0
        )
    }
}

/// Initializer is the buyer and is buying an nft from the seller
/// The seller is the owner of the order account
/// Buyer transfers sol to seller account
#[inline(always)]
pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, FillOrder<'info>>) -> Result<()> {
    let nft_token_program_key = &ctx.accounts.nft_token_program.key.to_string().clone();
    let nft_program_key = &ctx.accounts.nft_program.key.to_string().clone();

    let remaining_accounts = ctx.remaining_accounts.to_vec();

    let bump = &get_bump_in_seed_form(&ctx.bumps.order);

    let signer_seeds: &[&[&[u8]]; 1] = &[&[ORDER_SEED, ctx.accounts.order.nonce.as_ref(), ctx.accounts.order.market.as_ref(), ctx.accounts.order.owner.as_ref(), bump][..]];

    let buy_price = ctx.accounts.order.price;
    let _fee_amount = get_fee_amount(buy_price);

    let is_buy = ctx.accounts.order.side == 0;
    // Verify maker + taker accounts
    // Verify the buyer account
    if is_buy {
        require!(ctx.accounts.buyer.key() == ctx.accounts.maker.key(), MarketError::WrongAccount);
        require!(ctx.accounts.seller.key() == ctx.accounts.taker.key(), MarketError::WrongAccount);
    } else {
        require!(ctx.accounts.buyer.key() == ctx.accounts.taker.key(), MarketError::WrongAccount);
        require!(ctx.accounts.seller.key() == ctx.accounts.order.key(), MarketError::WrongAccount);
    }
    // Transfer payment
    if is_buy {
        // Transfer funds -- BUY ORDER
        // transfer from order to seller
        
    } else {
        // Transfer funds -- SELL ORDER
        // transfer from buyer to seller

    }

    // Transfer NFT
    if *nft_token_program_key == TOKEN_PID {
        // Check if its metaplex or not
        if *nft_program_key == METAPLEX_PID {
            // If it's metaplex, we parse the first remaining account as nft_metadata
            let nft_metadata = remaining_accounts.get(0).unwrap();
            let nft_edition = remaining_accounts.get(1).unwrap();

            // The remaining metadata accounts are (PNFT ONLY):
                // 0 owner_token_record or default,
                // 1 authorization_rules or default,
                // 2 authorization_rules_program or default,
                // 3 destination_token_record or default,
                // 4 delegate record or default,
                // 5 existing delegate or default,
                // 6 existing delegate record or default

            let (_, extra_remaining_accounts) = remaining_accounts.split_at(2);

            let parsed_accounts = parse_remaining_accounts_pnft(extra_remaining_accounts.to_vec(), true, None);
            let pnft_params = parsed_accounts.pnft_params;

            let parsed_metadata = Metadata::safe_deserialize(&nft_metadata.data.borrow())?;
            let is_pnft = get_is_pnft(&parsed_metadata);
    
            let extra_accounts = ExtraTransferParams {
                owner_token_record: pnft_params.owner_token_record,
                dest_token_record: pnft_params.destination_token_record,
                authorization_rules: pnft_params.authorization_rules,
                authorization_data: None,
                authorization_rules_program: pnft_params.authorization_rules_program,
            };

            let transfer_params = MetaplexAdditionalTransferAccounts {
                metadata: nft_metadata.to_account_info(),
                edition: nft_edition.to_account_info(),
                extra_accounts
            };

            ctx.accounts.metaplex_nft_transfer(is_buy, is_pnft, transfer_params)?;

            if is_pnft {
                // TODO
                pay_royalties(
                    buy_price,
                    nft_metadata.to_account_info().clone(),
                    ctx.accounts.taker.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                    parsed_accounts.creator_accounts,
                    false,
                    Some(signer_seeds),
                )?;
            }
        } else {
            // Transfer compressed NFT
            // TODO
            return Err(MarketError::UnsupportedNft.into())
        }
    } else if *nft_token_program_key == TOKEN_EXT_PID {
        let mut token22_ra = remaining_accounts.clone();
        // Check if its WNS
        if *nft_program_key == WNS_PID {
            // Remaining Accounts 0-2 for approval
            let approval_account = remaining_accounts.get(0).unwrap();
            let distribution_account = remaining_accounts.get(1).unwrap();
            let distribution_program = remaining_accounts.get(2).unwrap();
            let (_, extra_remaining_accounts) = remaining_accounts.split_at(3);
            token22_ra = extra_remaining_accounts.to_vec();

            let wns_accounts = WnsApprovalAccounts {
                approval_account: approval_account.to_account_info(),
                distribution_account: distribution_account.to_account_info(),
                distribution_program: distribution_program.to_account_info(),
            };
            // Handles royalties
            ctx.accounts.approve_wns_transfer(buy_price, wns_accounts)?;
        }
        // Any remaining accounts left are for potential transfer hook (Empty if not expecting hook) 
        ctx.accounts.token22_nft_transfer(is_buy, token22_ra)?;
    } else if *nft_token_program_key == BUBBLEGUM_PID {
        // Transfer compressed NFT
        // TODO
        return Err(MarketError::UnsupportedNft.into())
    } else {
        // ERROR
        return Err(MarketError::UnsupportedNft.into())
    }

    // close order account
    let size = ctx.accounts.order.size;
    let clock = Clock::get()?;

    if is_buy {
        Order::edit_buy(
            &mut ctx.accounts.order,
            buy_price,
            size - 1,
            clock.unix_timestamp,
        );
        if size == 1 {
            ctx.accounts.order.state = OrderState::Closed.into();
            emit_cpi!(Order::get_edit_event(
                &mut ctx.accounts.order.clone(),
                ctx.accounts.order.key(),
                ctx.accounts.market.market_identifier,
                OrderEditType::FillAndClose,
            ));
            ctx.accounts
                .order
                .close(ctx.accounts.maker.to_account_info())?;
        }
    } else {
        ctx.accounts.order.state = OrderState::Closed.into();
        emit_cpi!(Order::get_edit_event(
            &mut ctx.accounts.order.clone(),
            ctx.accounts.order.key(),
            ctx.accounts.market.market_identifier,
            OrderEditType::FillAndClose,
        ));
        ctx.accounts
            .order
            .close(ctx.accounts.maker.to_account_info())?;
    }

    Ok(())
}
