use anchor_lang::{prelude::*, solana_program::sysvar};
use anchor_spl::{
    associated_token::AssociatedToken, token_2022::TransferChecked as Token22TransferChecked, token_interface::{
        Mint, TokenAccount, TokenInterface
    }
};
use mpl_token_metadata::accounts::Metadata;
use wen_new_standard::cpi::{accounts::ApproveTransfer, approve_transfer};

use crate::{
    errors::MarketError, state::*, utils::{metaplex::pnft::utils::get_is_pnft, mplx_transfer::{transfer_metaplex_nft, ExtraTransferParams, MetaplexAdditionalTransferAccounts, TransferMetaplexNft}, parse_remaining_accounts_pnft, token_extensions::{transfer_token22_checked, WnsApprovalAccounts}}
};

#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct ListData {
    pub nonce: Pubkey,
    pub price: u64,
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
        seeds = [VERIFICATION_SEED, market.market_identifier.as_ref(), nft_mint.key().as_ref()],
        bump,
        constraint = verification.verified == 1
    )]
    pub verification: Box<Account<'info, MintVerification>>,
    #[account(
        mint::token_program = nft_token_program
    )]
    pub nft_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,
        constraint = initializer_nft_ta.owner == initializer.key(),
        constraint = initializer_nft_ta.mint == nft_mint.key(),
    )]
    pub initializer_nft_ta: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = order_nft_ta.owner == order.key(),
        constraint = order_nft_ta.mint == nft_mint.key(),
    )]
    pub order_nft_ta: Box<InterfaceAccount<'info, TokenAccount>>,
    /// CHECK: checked by constraint and in cpi
    #[account(address = sysvar::instructions::id())]
    pub sysvar_instructions: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub payment_token_program: Interface<'info, TokenInterface>,
    /// CHECK: checked by constraint and in cpi
    pub nft_token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// CHECK: checked by constraint and in cpi
    pub nft_program: UncheckedAccount<'info>,
}

impl<'info> ListNft<'info> {

    /*
        Metaplex Transfer Instructions
    */ 

    fn metaplex_nft_transfer(&self, is_pnft: bool, extra_transfer_accounts: MetaplexAdditionalTransferAccounts<'info>) -> Result<()> {
        let transfer_accounts = TransferMetaplexNft {
            authority: self.initializer.to_account_info(),
            payer: self.initializer.to_account_info(),
            source_owner: self.initializer.to_account_info(),
            source_ta: self.initializer_nft_ta.to_account_info(),
            destination_owner: self.order.to_account_info(),
            destination_ta: self.order_nft_ta.to_account_info(),
            mint: self.nft_mint.to_account_info(),
            metadata: extra_transfer_accounts.metadata.to_account_info(),
            edition: extra_transfer_accounts.edition.to_account_info(),
            system_program: self.system_program.to_account_info(),
            instructions: self.sysvar_instructions.to_account_info(),
            token_program: self.nft_token_program.to_account_info(),
            ata_program: self.associated_token_program.to_account_info(),
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
    fn approve_wns_transfer(&self, wns_accounts: WnsApprovalAccounts<'info>) -> Result<()> {
        let cpi_program = self.nft_program.to_account_info();
        let cpi_accounts = ApproveTransfer {
            payer: self.initializer.to_account_info(),
            authority: self.initializer.to_account_info(),
            mint: self.nft_mint.to_account_info(),
            approve_account: wns_accounts.approval_account.to_account_info(),
            payment_mint: self.system_program.to_account_info(), //  wont be used
            distribution_token_account: None, // wont be used
            authority_token_account: None, // wont be used
            distribution_account: wns_accounts.distribution_account.to_account_info(),
            system_program: self.system_program.to_account_info(),
            distribution_program: wns_accounts.distribution_program.to_account_info(),
            token_program: self.nft_token_program.to_account_info(),
            payment_token_program: None,
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        approve_transfer(cpi_ctx, 0)
    }

    // General Token22 Transfer
    fn token22_nft_transfer(&self, remaining_accounts: Vec<AccountInfo<'info>>) -> Result<()> {
        let transfer_cpi = CpiContext::new(
            self.nft_token_program.to_account_info(),
            Token22TransferChecked {
                from: self.initializer_nft_ta.to_account_info(),
                to: self.order_nft_ta.to_account_info(),
                authority: self.initializer.to_account_info(),
                mint: self.nft_mint.to_account_info(),
            },
        );

        transfer_token22_checked(
            transfer_cpi.with_remaining_accounts(remaining_accounts),
            1, // supply = 1
            0, // decimals = 0
        )
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
        clock.unix_timestamp,
        OrderSide::Sell.into(),
        1, // always 1
        data.price,
        OrderState::Ready.into(),
        true,
    );

    // NFT Transfer
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

            ctx.accounts.metaplex_nft_transfer(is_pnft, transfer_params)?;
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
            ctx.accounts.approve_wns_transfer(wns_accounts)?;
        }
        // Any remaining accounts left are for potential transfer hook (Empty if not expecting hook) 
        ctx.accounts.token22_nft_transfer(token22_ra)?;
    } else if *nft_token_program_key == BUBBLEGUM_PID {
        // Transfer compressed NFT
        // TODO
        return Err(MarketError::UnsupportedNft.into())
    } else {
        // ERROR
        return Err(MarketError::UnsupportedNft.into())
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
