use anchor_lang::{prelude::*, solana_program::sysvar};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        approve, freeze_account, Approve, FreezeAccount, Mint, TokenAccount, TokenInterface,
    },
};
use wen_new_standard::cpi::{accounts::FreezeDelegatedAccount, freeze_mint_account};
use mpl_token_metadata::accounts::Metadata;

// use spl_token_group_interface::state::TokenGroupMember;

use crate::{
    errors::MarketError,
    state::*,
    utils::{
        get_bump_in_seed_form, metaplex::pnft::utils::get_is_pnft, mplx_transfer::{ExtraTransferParams, MetaplexAdditionalTransferAccounts}, parse_remaining_accounts_pnft, verify_wns_mint
    },
};

#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct ListData {
    pub nonce: Pubkey,
    pub payment_mint: Pubkey,
    pub price: u64,
    pub size: u64,
}

#[derive(Accounts)]
#[instruction(data: ListData)]
#[event_cpi]
pub struct ListNft<'info> {
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
        constraint = data.price > 0,
        constraint = data.size > 0,
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
        mint::token_program = nft_token_program
    )]
    pub nft_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,
        associated_token::mint = nft_mint,
        associated_token::authority = initializer,
        associated_token::token_program = nft_token_program
    )]
    pub initializer_nft_ta: Box<InterfaceAccount<'info, TokenAccount>>,
    /// CHECK: checked by constraint and in cpi
    #[account(address = sysvar::instructions::id())]
    pub sysvar_instructions: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    /// CHECK: checked by constraint and in cpi
    pub nft_token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// CHECK: checked by constraint and in cpi
    pub nft_program: UncheckedAccount<'info>,
}

impl<'info> ListNft<'info> {
    /*
        Metaplex Delegate Instructions
    */

    /*
        Compressed Delegate Instructions
    */

    /*
        Token 22 Delegate Instructions
    */
    fn token22_nft_delegate(
        &self,
        size: u64,
        remaining_accounts: Vec<AccountInfo<'info>>,
    ) -> Result<()> {
        let delegate_cpi = CpiContext::new(
            self.nft_token_program.to_account_info(),
            Approve {
                to: self.initializer_nft_ta.to_account_info(),
                authority: self.initializer.to_account_info(),
                delegate: self.order.to_account_info(),
            },
        );

        approve(
            delegate_cpi.with_remaining_accounts(remaining_accounts),
            size, // supply = 1
        )
    }

    fn wns_freeze(
        &self,
        signer_seeds: &[&[&[u8]]],
        manager_account: AccountInfo<'info>,
        remaining_accounts: Vec<AccountInfo<'info>>,
    ) -> Result<()> {
        
        let freeze_cpi = CpiContext::new_with_signer(
            self.nft_program.to_account_info(),
            FreezeDelegatedAccount {
                mint: self.nft_mint.to_account_info(),
                user: self.initializer.to_account_info(),
                delegate_authority: self.order.to_account_info(),
                mint_token_account: self.initializer_nft_ta.to_account_info(),
                manager: manager_account.to_account_info(),
                token_program: self.nft_token_program.to_account_info(),
            },
            signer_seeds
        );

        freeze_mint_account(
            freeze_cpi.with_remaining_accounts(remaining_accounts)
        )
    }

    fn verify_mint(&self, verification_account: &AccountInfo<'info>) -> Result<()> {
        if verification_account.key()
            != get_verification_pda(self.nft_mint.key(), self.market.key()).0
        {
            return Err(MarketError::InvalidNft.into());
        }
        let verification =
            MintVerification::deserialize(&mut &verification_account.data.borrow_mut()[8..])?;

        if verification.verified != 1 {
            return Err(MarketError::InvalidNft.into());
        }

        Ok(())
    }
}

#[inline(always)]
pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, ListNft<'info>>,
    data: ListData,
) -> Result<()> {
    msg!("Initialize a new sell order: {}", ctx.accounts.order.key());

    let nft_token_program_key = &ctx.accounts.nft_token_program.key.to_string().clone();
    let nft_program_key = &ctx.accounts.nft_program.key.to_string().clone();
    let remaining_accounts = ctx.remaining_accounts.to_vec();

    let clock = Clock::get()?;
    // create a new order with size 1
    Order::init(
        &mut ctx.accounts.order,
        ctx.accounts.market.key(),
        ctx.accounts.initializer.key(),
        data.nonce,
        ctx.accounts.nft_mint.key(),
        data.payment_mint,
        clock.unix_timestamp,
        OrderSide::Sell.into(),
        data.size, // always 1
        data.price,
        OrderState::Ready.into(),
        true,
    );

    let bump = &get_bump_in_seed_form(&ctx.bumps.order);
    let signer_seeds: &[&[&[u8]]; 1] = &[&[
        ORDER_SEED,
        ctx.accounts.order.nonce.as_ref(),
        ctx.accounts.order.market.as_ref(),
        ctx.accounts.order.owner.as_ref(),
        bump,
    ][..]];
    // NFT Transfer
    if *nft_token_program_key == TOKEN_PID {
        // Check if its metaplex or not
        if *nft_program_key == METAPLEX_PID {
            // If it's metaplex, we parse the first remaining account as nft_metadata
            let nft_metadata = remaining_accounts.get(0).unwrap();
            let nft_edition = remaining_accounts.get(1).unwrap();

            // Must use manual verify for Metaplex
            let verification_account = remaining_accounts.get(2).unwrap();

            ctx.accounts.verify_mint(verification_account)?;

            // The remaining metadata accounts are (PNFT ONLY):
            // 0 owner_token_record or default,
            // 1 authorization_rules or default,
            // 2 authorization_rules_program or default,
            // 3 destination_token_record or default,
            // 4 delegate record or default,
            // 5 existing delegate or default,
            // 6 existing delegate record or default

            let (_, extra_remaining_accounts) = remaining_accounts.split_at(3);

            let parsed_accounts =
                parse_remaining_accounts_pnft(extra_remaining_accounts.to_vec(), true, None);
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
                extra_accounts,
            };
            // TODO Delegate Metaplex NFT
            return Err(MarketError::UnsupportedNft.into());
            // ctx.accounts.metaplex_nft_transfer(is_pnft, transfer_params)?;
        } else {
            // Transfer compressed NFT
            // TODO
            return Err(MarketError::UnsupportedNft.into());
        }
    } else if *nft_token_program_key == TOKEN_EXT_PID {
        let mut token22_ra = remaining_accounts.clone();
        // Pass in RA for delegate as needed
        ctx.accounts.token22_nft_delegate(data.size, token22_ra.clone())?;
        if *nft_program_key == WNS_PID {
            let group_member_account = remaining_accounts.get(4).unwrap();
            let manager_account = remaining_accounts.get(6).unwrap();
            
            let (_, extra_remaining_accounts) = remaining_accounts.split_at(7);
            token22_ra = extra_remaining_accounts.to_vec();
            
            verify_wns_mint(ctx.accounts.nft_mint.to_account_info(),group_member_account.to_account_info(), ctx.accounts.market.market_identifier.clone())?;
            ctx.accounts.wns_freeze(signer_seeds, manager_account.to_account_info(), token22_ra.clone())?;
        }
    } else if *nft_token_program_key == BUBBLEGUM_PID {
        // Transfer compressed NFT
        // TODO
        return Err(MarketError::UnsupportedNft.into());
    } else {
        // ERROR
        return Err(MarketError::UnsupportedNft.into());
    }

    // Emit event
    emit_cpi!(Order::get_edit_event(
        &mut ctx.accounts.order.clone(),
        ctx.accounts.order.key(),
        ctx.accounts.market.market_identifier,
        OrderEditType::Init,
    ));

    Ok(())
}
