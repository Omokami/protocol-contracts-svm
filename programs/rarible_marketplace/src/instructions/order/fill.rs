use std::collections::HashMap;
use std::str::FromStr;

use anchor_lang::{prelude::*, solana_program::sysvar};
use anchor_spl::{
    associated_token::{get_associated_token_address, AssociatedToken},
    token_interface::{
        approve, revoke, transfer_checked, Approve, Mint, Revoke, TokenInterface, TransferChecked,
    },
};
use mpl_token_metadata::accounts::Metadata;
use wen_new_standard::{
    cpi::{accounts::{ApproveTransfer, ThawDelegatedAccount}, approve_transfer, thaw_mint_account},
    utils::get_mint_metadata,
    ROYALTY_BASIS_POINTS_FIELD,
};

use crate::{
    errors::MarketError,
    state::*,
    utils::{
        create_ata, get_amount_from_bp, get_bump_in_seed_form, get_fee_amount,
        metaplex::pnft::utils::get_is_pnft,
        mplx_transfer::{
            transfer_metaplex_nft, ExtraTransferParams, MetaplexAdditionalTransferAccounts,
            TransferMetaplexNft,
        },
        parse_remaining_accounts_pnft,
        token_extensions::WnsApprovalAccounts,
        validate_associated_token_account, verify_wns_mint,
    },
};

#[derive(Accounts)]
#[instruction()]
#[event_cpi]
pub struct FillOrder<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,
    #[account(mut, constraint = maker.key() == order.owner.key())]
    /// CHECK: constraint check
    pub maker: UncheckedAccount<'info>,
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
        seeds = [VERIFICATION_SEED, nft_mint.key().as_ref(), market.key().as_ref()],
        bump,
        constraint = verification.verified == 1
    )]
    pub verification: Box<Account<'info, MintVerification>>,
    #[account(mut)]
    /// CHECK: checked by create_ata function
    pub seller_nft_ta: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: checked by create_ata function
    pub buyer_nft_ta: UncheckedAccount<'info>,
    #[account(mut, constraint = fee_recipient.key() == market.fee_recipient.key())]
    /// CHECK: constraint check
    pub fee_recipient: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: checked by create_ata function
    pub fee_recipient_ta: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub nft_token_program: Interface<'info, TokenInterface>,
    /// CHECK: checked by constraint and in cpi
    pub nft_program: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: checked by create_ata function
    pub seller_payment_ta: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: checked by create_ata function
    pub buyer_payment_ta: UncheckedAccount<'info>,
    #[account(mut, constraint = payment_mint.key() == order.payment_mint)]
    pub payment_mint: Box<InterfaceAccount<'info, Mint>>,
    pub payment_token_program: Interface<'info, TokenInterface>,
    /// CHECK: checked by constraint and in cpi
    #[account(address = sysvar::instructions::id())]
    pub sysvar_instructions: UncheckedAccount<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> FillOrder<'info> {
    #[inline(never)]
    fn transfer_payment(&self, signer_seeds: &[&[&[u8]]], is_buy: bool, amount: u64) -> Result<()> {
        let cpi_ctx = if is_buy {
            CpiContext::new_with_signer(
                self.payment_token_program.to_account_info(),
                TransferChecked {
                    from: self.buyer_payment_ta.to_account_info(),
                    to: self.seller_payment_ta.to_account_info(),
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
                    to: self.seller_payment_ta.to_account_info(),
                    authority: self.taker.to_account_info(),
                    mint: self.payment_mint.to_account_info(),
                },
            )
        };
        transfer_checked(cpi_ctx, amount, self.payment_mint.decimals)
    }

    #[inline(never)]
    fn transfer_royalty(
        &self,
        signer_seeds: &[&[&[u8]]],
        creator_token_account: &AccountInfo<'info>,
        is_buy: bool,
        amount: u64,
    ) -> Result<()> {
        let cpi_ctx = if is_buy {
            CpiContext::new_with_signer(
                self.payment_token_program.to_account_info(),
                TransferChecked {
                    from: self.buyer_payment_ta.to_account_info(),
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
                },
            )
        };
        transfer_checked(cpi_ctx, amount, self.payment_mint.decimals)
    }

    fn wns_thaw(
        &self,
        signer_seeds: &[&[&[u8]]],
        seller_account: AccountInfo<'info>,
        manager_account: AccountInfo<'info>,
        remaining_accounts: Vec<AccountInfo<'info>>,
    ) -> Result<()> {
        
        let thaw_cpi = CpiContext::new_with_signer(
            self.nft_program.to_account_info(),
            ThawDelegatedAccount {
                mint: self.nft_mint.to_account_info(),
                user: seller_account.to_account_info(),
                delegate_authority: self.order.to_account_info(),
                mint_token_account: self.seller_nft_ta.to_account_info(),
                manager: manager_account.to_account_info(),
                token_program: self.nft_token_program.to_account_info(),
            },
            signer_seeds
        );

        thaw_mint_account(
            thaw_cpi.with_remaining_accounts(remaining_accounts)
        )
    }

    /*
        Metaplex Transfer Instructions
    */
    #[inline(never)]
    fn metaplex_nft_transfer(
        &self,
        is_buy: bool,
        is_pnft: bool,
        extra_transfer_accounts: MetaplexAdditionalTransferAccounts<'info>,
    ) -> Result<()> {
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
            is_pnft,
        )
    }

    /*
        Compressed Transfer Instructions
    */

    /*
        Token 22 Transfer Instructions
    */

    // WNS Pre-Transfer Approval
    #[inline(never)]
    fn approve_wns_transfer(
        &self,
        signer_seeds: &[&[&[u8]]],
        buy_amount: u64,
        is_buy: bool,
        wns_accounts: WnsApprovalAccounts<'info>,
    ) -> Result<()> {
        let cpi_program = self.nft_program.to_account_info();
        create_ata(
            &wns_accounts.distribution_token_account.to_account_info(),
            &self.taker.to_account_info(),
            &self.payment_mint.to_account_info(),
            &wns_accounts.distribution_account.to_account_info(),
            &self.system_program.to_account_info(),
            &self.payment_token_program.to_account_info(),
        )?;
        let cpi_ctx = if is_buy {
            let cpi_accounts = ApproveTransfer {
                payer: self.taker.to_account_info(),
                authority: self.order.to_account_info(),
                mint: self.nft_mint.to_account_info(),
                approve_account: wns_accounts.approval_account.to_account_info(),
                payment_mint: self.payment_mint.to_account_info(),
                distribution_token_account: Some(
                    wns_accounts.distribution_token_account.to_account_info(),
                ),
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
                payment_mint: self.payment_mint.to_account_info(),
                distribution_token_account: Some(
                    wns_accounts.distribution_token_account.to_account_info(),
                ),
                authority_token_account: Some(self.buyer_payment_ta.to_account_info()),
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
    #[inline(never)]
    fn token22_nft_transfer(
        &self,
        signer_seeds: &[&[&[u8]]],
        is_buy: bool,
        amount: u64,
        remaining_accounts: Vec<AccountInfo<'info>>,
    ) -> Result<()> {
        let transfer_cpi = if is_buy {
            CpiContext::new(
                self.nft_token_program.to_account_info(),
                TransferChecked {
                    from: self.seller_nft_ta.to_account_info(),
                    to: self.buyer_nft_ta.to_account_info(),
                    authority: self.taker.to_account_info(),
                    mint: self.nft_mint.to_account_info(),
                },
            )
        } else {
            CpiContext::new_with_signer(
                self.nft_token_program.to_account_info(),
                TransferChecked {
                    from: self.seller_nft_ta.to_account_info(),
                    to: self.buyer_nft_ta.to_account_info(),
                    authority: self.order.to_account_info(),
                    mint: self.nft_mint.to_account_info(),
                },
                signer_seeds,
            )
        };

        if !is_buy {
            let revoke_cpi = CpiContext::new_with_signer(
                self.nft_token_program.to_account_info(),
                Revoke {
                    source: self.seller_nft_ta.to_account_info(),
                    authority: self.order.to_account_info(),
                },
                signer_seeds,
            );
            revoke(revoke_cpi)?;
            transfer_checked(
                transfer_cpi.with_remaining_accounts(remaining_accounts),
                amount, // supply = 1
                0,      // decimals = 0
            )
        } else {
            transfer_checked(
                transfer_cpi.with_remaining_accounts(remaining_accounts),
                amount, // supply = 1
                0,      // decimals = 0
            )
        }
    }

    #[inline(never)]
    fn transfer_fee(&self, signer_seeds: &[&[&[u8]]], is_buy: bool, amount: u64) -> Result<()> {
        let cpi_ctx = if is_buy {
            CpiContext::new_with_signer(
                self.payment_token_program.to_account_info(),
                TransferChecked {
                    from: self.buyer_payment_ta.to_account_info(),
                    to: self.fee_recipient_ta.to_account_info(),
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
                    to: self.fee_recipient_ta.to_account_info(),
                    authority: self.taker.to_account_info(),
                    mint: self.payment_mint.to_account_info(),
                },
            )
        };
        transfer_checked(cpi_ctx, amount, self.payment_mint.decimals)
    }

    fn revoke_payment_account(&self, signer_seeds: &[&[&[u8]]], is_buy: bool) -> Result<()> {
        if is_buy {
            let cpi_ctx = CpiContext::new_with_signer(
                self.payment_token_program.to_account_info(),
                Revoke {
                    authority: self.order.to_account_info(),
                    source: self.buyer_payment_ta.to_account_info(),
                },
                signer_seeds,
            );
            revoke(cpi_ctx)
        } else {
            Ok(())
        }
    }
}

/// Initializer is the buyer and is buying an nft from the seller
/// The seller is the owner of the order account
/// Buyer transfers sol to seller account
#[inline(never)]
pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, FillOrder<'info>>,
    amount: u64,
) -> Result<()> {
    let nft_token_program_key = &ctx.accounts.nft_token_program.key.to_string().clone();
    let nft_program_key = &ctx.accounts.nft_program.key.to_string().clone();

    let remaining_accounts = ctx.remaining_accounts.to_vec();

    let bump = &get_bump_in_seed_form(&ctx.bumps.order);

    let signer_seeds: &[&[&[u8]]; 1] = &[&[
        ORDER_SEED,
        ctx.accounts.order.nonce.as_ref(),
        ctx.accounts.order.market.as_ref(),
        ctx.accounts.order.owner.as_ref(),
        bump,
    ][..]];

    let buy_price = ctx.accounts.order.price;
    let order_size = ctx.accounts.order.size;

    if order_size < amount {
        return Err(MarketError::InsufficientOrderSize.into());
    }

    let buy_value = amount.checked_mul(buy_price).unwrap();

    let fee_amount = get_fee_amount(buy_value, ctx.accounts.market.fee_bps);

    let mut seller_received_amount = buy_value - fee_amount;

    let is_buy = ctx.accounts.order.side == 0;
    // Verify maker + taker accounts
    // Verify the buyer account

    let system_program = ctx.accounts.system_program.to_account_info();
    let nft_token_program = ctx.accounts.nft_token_program.to_account_info();
    let payment_token_program = ctx.accounts.payment_token_program.to_account_info();

    let nft_funder = if is_buy {
        ctx.accounts.taker.to_account_info()
    } else {
        ctx.accounts.maker.to_account_info()
    };
    create_ata(
        &ctx.accounts.seller_nft_ta.to_account_info(),
        &ctx.accounts.taker.to_account_info(),
        &ctx.accounts.nft_mint.to_account_info(),
        &nft_funder,
        &system_program,
        &nft_token_program,
    )?;

    let nft_receiver = if is_buy {
        ctx.accounts.maker.to_account_info()
    } else {
        ctx.accounts.taker.to_account_info()
    };
    create_ata(
        &ctx.accounts.buyer_nft_ta.to_account_info(),
        &ctx.accounts.taker.to_account_info(),
        &ctx.accounts.nft_mint.to_account_info(),
        &nft_receiver,
        &system_program,
        &nft_token_program,
    )?;

    let payment_funder = if is_buy {
        ctx.accounts.maker.to_account_info()
    } else {
        ctx.accounts.taker.to_account_info()
    };
    create_ata(
        &ctx.accounts.buyer_payment_ta.to_account_info(),
        &ctx.accounts.taker.to_account_info(),
        &ctx.accounts.payment_mint.to_account_info(),
        &payment_funder,
        &system_program,
        &payment_token_program,
    )?;

    let payment_receiver = if is_buy {
        ctx.accounts.taker.to_account_info()
    } else {
        ctx.accounts.maker.to_account_info()
    };
    create_ata(
        &ctx.accounts.seller_payment_ta.to_account_info(),
        &ctx.accounts.taker.to_account_info(),
        &ctx.accounts.payment_mint.to_account_info(),
        &payment_receiver,
        &system_program,
        &payment_token_program,
    )?;

    let fee_reciever = ctx.accounts.fee_recipient.to_account_info();
    create_ata(
        &ctx.accounts.fee_recipient_ta.to_account_info(),
        &ctx.accounts.taker.to_account_info(),
        &ctx.accounts.payment_mint.to_account_info(),
        &fee_reciever,
        &system_program,
        &payment_token_program,
    )?;

    // Transfer NFT
    if *nft_token_program_key == TOKEN_PID {
        // Check if its metaplex or not
        if *nft_program_key == METAPLEX_PID {
            // TODO
            return Err(MarketError::UnsupportedNft.into());
        } else {
            // Transfer compressed NFT
            // TODO
            return Err(MarketError::UnsupportedNft.into());
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
            let group_member_account = remaining_accounts.get(4).unwrap();
            let payment_mint = remaining_accounts.get(5).unwrap();

            verify_wns_mint(ctx.accounts.nft_mint.to_account_info(),group_member_account.to_account_info(), ctx.accounts.market.market_identifier.clone())?;

            let (_, extra_remaining_accounts) = remaining_accounts.split_at(6);
            token22_ra = extra_remaining_accounts.to_vec();

            let wns_accounts = WnsApprovalAccounts {
                approval_account: approval_account.to_account_info(),
                distribution_account: distribution_account.to_account_info(),
                distribution_token_account: distribution_token_account.to_account_info(),
                distribution_program: distribution_program.to_account_info(),
                payment_mint: payment_mint.to_account_info(),
            };

            let mint_metadata = get_mint_metadata(&mut ctx.accounts.nft_mint.to_account_info())?;
            let royalty_basis_points = mint_metadata
                .additional_metadata
                .iter()
                .find(|(key, _)| key == ROYALTY_BASIS_POINTS_FIELD)
                .map(|(_, value)| value)
                .map(|value| u64::from_str(value).unwrap())
                .unwrap_or(0);

            let royalties = get_amount_from_bp(buy_value, royalty_basis_points.into())?;
            seller_received_amount = seller_received_amount.checked_sub(royalties).unwrap();

            // Handles royalties
            ctx.accounts
                .approve_wns_transfer(signer_seeds, buy_value, is_buy, wns_accounts)?;
        }
        // Any remaining accounts left are for potential transfer hook (Empty if not expecting hook)
        ctx.accounts
            .token22_nft_transfer(signer_seeds, is_buy, order_size, token22_ra)?;
    } else if *nft_token_program_key == BUBBLEGUM_PID {
        // Transfer compressed NFT
        // TODO
        return Err(MarketError::UnsupportedNft.into());
    } else {
        // ERROR
        return Err(MarketError::UnsupportedNft.into());
    }

    // Transfer payment
    ctx.accounts
        .transfer_payment(signer_seeds, is_buy, seller_received_amount)?;
    ctx.accounts
        .transfer_fee(signer_seeds, is_buy, fee_amount)?;
    ctx.accounts.revoke_payment_account(signer_seeds, is_buy)?;
    // close order account
    let size = ctx.accounts.order.size;
    let payment_mint = ctx.accounts.order.payment_mint;
    let clock = Clock::get()?;

    let new_size = size - amount;
    Order::edit_order(
        &mut ctx.accounts.order,
        buy_price,
        payment_mint,
        new_size,
        clock.unix_timestamp,
    );
    if new_size == 0 {
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
