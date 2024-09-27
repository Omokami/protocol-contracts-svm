use std::collections::HashMap;
use std::str::FromStr;

use anchor_lang::{prelude::*, solana_program::sysvar};
use anchor_spl::{
    associated_token::{get_associated_token_address, AssociatedToken}, token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked}
};
use mpl_token_metadata::accounts::Metadata;
use wen_new_standard::{cpi::{accounts::ApproveTransfer, approve_transfer}, utils::get_mint_metadata, ROYALTY_BASIS_POINTS_FIELD};

use crate::{
    errors::MarketError,
    state::*,
    utils::{
        get_amount_from_bp, get_bump_in_seed_form, get_fee_amount, metaplex::pnft::utils::get_is_pnft, mplx_transfer::{transfer_metaplex_nft, ExtraTransferParams, MetaplexAdditionalTransferAccounts, TransferMetaplexNft}, parse_remaining_accounts_pnft, token_extensions::{transfer_token22_checked, WnsApprovalAccounts}, validate_associated_token_account
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
    #[account(
        associated_token::mint = payment_mint,
        associated_token::authority = seller,
        associated_token::token_program = payment_token_program,
    )]
    pub seller_payment_ta: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = payment_mint,
        associated_token::authority = buyer,
        associated_token::token_program = payment_token_program,
    )]
    pub buyer_payment_ta: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut, constraint = payment_mint.key() == order.payment_mint)]
    pub payment_mint: Box<InterfaceAccount<'info, Mint>>,
    pub payment_token_program: Interface<'info, TokenInterface>,
    /// CHECK: checked by constraint and in cpi
    #[account(address = sysvar::instructions::id())]
    pub sysvar_instructions: UncheckedAccount<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// CHECK: checked by constraint and in cpi
    pub token_metadata_program: UncheckedAccount<'info>,
}

impl<'info> FillOrder<'info> {

    fn transfer_payment(&self, signer_seeds: &[&[&[u8]]], is_buy: bool, amount: u64) -> Result<()> {
        let cpi_ctx = if is_buy {
            CpiContext::new_with_signer(
                self.payment_token_program.to_account_info(),
                TransferChecked {
                    from: self.seller_payment_ta.to_account_info(),
                    to: self.buyer_payment_ta.to_account_info(),
                    authority: self.order.to_account_info(),
                    mint: self.payment_mint.to_account_info(),
                },
                signer_seeds,
            )
        } else {
            CpiContext::new(
                self.payment_token_program.to_account_info(),
                TransferChecked {
                    from: self.seller_payment_ta.to_account_info(),
                    to: self.buyer_payment_ta.to_account_info(),
                    authority: self.taker.to_account_info(),
                    mint: self.payment_mint.to_account_info(),
                }
            )
        };
        transfer_checked(cpi_ctx, amount, self.payment_mint.decimals)
    }

    fn transfer_royalty(&self, signer_seeds: &[&[&[u8]]], creator_token_account: &AccountInfo<'info>, is_buy: bool, amount: u64) -> Result<()> {
        let cpi_ctx = if is_buy {
            CpiContext::new_with_signer(
                self.payment_token_program.to_account_info(),
                TransferChecked {
                    from: self.seller_payment_ta.to_account_info(),
                    to: creator_token_account.to_account_info(),
                    authority: self.order.to_account_info(),
                    mint: self.payment_mint.to_account_info(),
                },
                signer_seeds,
            )
        } else {
            CpiContext::new(
                self.payment_token_program.to_account_info(),
                TransferChecked {
                    from: self.buyer_payment_ta.to_account_info(),
                    to: creator_token_account.to_account_info(),
                    authority: self.taker.to_account_info(),
                    mint: self.payment_mint.to_account_info(),
                }
            )
        };
        transfer_checked(cpi_ctx, amount, self.payment_mint.decimals)
    }

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
    fn approve_wns_transfer(&self, signer_seeds: &[&[&[u8]]], buy_amount: u64, is_buy: bool, wns_accounts: WnsApprovalAccounts<'info>) -> Result<()> {
        let cpi_program = self.nft_program.to_account_info();
        let cpi_ctx = if is_buy {
            let cpi_accounts = ApproveTransfer {
                payer: self.taker.to_account_info(),
                authority: self.order.to_account_info(),
                mint: self.nft_mint.to_account_info(),
                approve_account: wns_accounts.approval_account.to_account_info(),
                payment_mint: self.payment_mint.to_account_info(),
                distribution_token_account: Some(wns_accounts.distribution_token_account.to_account_info()),
                authority_token_account: Some(self.buyer_payment_ta.to_account_info()),
                distribution_account: wns_accounts.distribution_account.to_account_info(),
                system_program: self.system_program.to_account_info(),
                distribution_program: wns_accounts.distribution_program.to_account_info(),
                token_program: self.nft_token_program.to_account_info(),
                payment_token_program: Some(self.payment_token_program.to_account_info()),
            };
            CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds)
        } else {
            let cpi_accounts = ApproveTransfer {
                payer: self.taker.to_account_info(),
                authority: self.taker.to_account_info(),
                mint: self.nft_mint.to_account_info(),
                approve_account: wns_accounts.approval_account.to_account_info(),
                payment_mint: self.system_program.to_account_info(),
                distribution_token_account: Some(wns_accounts.distribution_token_account.to_account_info()),
                authority_token_account: Some(self.taker.to_account_info()),
                distribution_account: wns_accounts.distribution_account.to_account_info(),
                system_program: self.system_program.to_account_info(),
                distribution_program: wns_accounts.distribution_program.to_account_info(),
                token_program: self.nft_token_program.to_account_info(),
                payment_token_program: Some(self.payment_token_program.to_account_info()),
            };
            CpiContext::new(cpi_program, cpi_accounts)
        };
        approve_transfer(cpi_ctx, buy_amount)
    }

    // General Token22 Transfer
    fn token22_nft_transfer(&self, is_buy: bool, remaining_accounts: Vec<AccountInfo<'info>>) -> Result<()> {
        let transfer_cpi = if is_buy {
            CpiContext::new(
                self.nft_token_program.to_account_info(),
                TransferChecked {
                    from: self.seller_nft_ta.to_account_info(),
                    to: self.buyer_nft_ta.to_account_info(),
                    authority: self.order.to_account_info(),
                    mint: self.nft_mint.to_account_info(),
                },
            )
        } else {
            CpiContext::new(
                self.nft_token_program.to_account_info(),
                TransferChecked {
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

    let mut seller_received_amount = buy_price - _fee_amount;

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
                let payment_program_key = ctx.accounts.payment_token_program.key();
                let payment_mint_key = ctx.accounts.payment_mint.key();
                let creator_accounts_map: HashMap<Pubkey, AccountInfo<'info>> = parsed_accounts.creator_token_accounts
                    .into_iter()
                    .map(|creator_token_account| (*creator_token_account.key, creator_token_account))
                    .collect();
                let royalties = get_amount_from_bp(buy_price, parsed_metadata.seller_fee_basis_points.into())?;
                seller_received_amount = seller_received_amount.checked_sub(royalties).unwrap();
                if let Some(creators) = parsed_metadata.creators.clone() {
                    for creator in creators {
                        if creator.share != 0 {
                            let amount = royalties
                                .checked_mul(creator.share.into())
                                .unwrap()
                                .checked_div(100)
                                .unwrap();
                            let expected_ata = get_associated_token_address(&creator.address, &payment_mint_key);
                            let ata = creator_accounts_map.get(&expected_ata);
                            if let Some(creator_ata) = ata {
                                validate_associated_token_account(creator_ata, &creator.address, &payment_mint_key, &payment_program_key)?;
                                ctx.accounts.transfer_royalty(signer_seeds, creator_ata, is_buy, amount)?;
                            }
                        }
                    }
                }
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
            let distribution_token_account = remaining_accounts.get(2).unwrap();
            let distribution_program = remaining_accounts.get(3).unwrap();
            let (_, extra_remaining_accounts) = remaining_accounts.split_at(4);
            token22_ra = extra_remaining_accounts.to_vec();

            let wns_accounts = WnsApprovalAccounts {
                approval_account: approval_account.to_account_info(),
                distribution_account: distribution_account.to_account_info(),
                distribution_token_account: distribution_token_account.to_account_info(),
                distribution_program: distribution_program.to_account_info(),
            };

            let mint_metadata = get_mint_metadata(&mut ctx.accounts.nft_mint.to_account_info())?;
            let royalty_basis_points = mint_metadata
                .additional_metadata
                .iter()
                .find(|(key, _)| key == ROYALTY_BASIS_POINTS_FIELD)
                .map(|(_, value)| value)
                .map(|value| u64::from_str(value).unwrap())
                .unwrap_or(0);

            let royalties = get_amount_from_bp(buy_price, royalty_basis_points.into())?;
            seller_received_amount = seller_received_amount.checked_sub(royalties).unwrap();
            
            // Handles royalties
            ctx.accounts.approve_wns_transfer(signer_seeds, buy_price, is_buy, wns_accounts)?;
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

    // Transfer payment
    ctx.accounts.transfer_payment(signer_seeds, is_buy, seller_received_amount)?;

    // close order account
    let size = ctx.accounts.order.size;
    let payment_mint = ctx.accounts.order.payment_mint;
    let clock = Clock::get()?;

    if is_buy {
        Order::edit_buy(
            &mut ctx.accounts.order,
            buy_price,
            payment_mint,
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
