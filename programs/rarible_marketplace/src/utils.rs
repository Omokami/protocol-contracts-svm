use std::collections::HashMap;

use anchor_lang::{
    prelude::{Account, AccountInfo, CpiContext, Error, Pubkey},
    solana_program::{
        program::{invoke, invoke_signed},
        system_instruction::transfer,
    },
    AccountDeserialize, ToAccountInfo,
};
use anchor_spl::token::TokenAccount;
use mpl_token_metadata::accounts::Metadata;
use program_utils::{
    bridgesplit_delegate, bridgesplit_freeze, bridgesplit_revoke, bridgesplit_thaw,
    bridgesplit_transfer,
    pnft::utils::{ExistingDelegateParams, PnftParams},
    BridgesplitDelegate, BridgesplitFreeze, BridgesplitRevoke, BridgesplitTransfer,
    ExtraDelegateParams, ExtraRevokeParams, ExtraTransferParams,
};
use vault::utils::{get_index_fee_bp, lamport_transfer};

use crate::state::{Order, PROTOCOL_FEES_BPS};

#[allow(clippy::too_many_arguments)]
pub fn transfer_nft<'info>(
    authority: AccountInfo<'info>,
    token_owner: AccountInfo<'info>,
    payer: AccountInfo<'info>,
    to: AccountInfo<'info>,
    nft_mint: AccountInfo<'info>,
    nft_metadata: AccountInfo<'info>,
    nft_edition: AccountInfo<'info>,
    from_nft_ta: AccountInfo<'info>,
    to_nft_ta: AccountInfo<'info>,
    system_program: AccountInfo<'info>,
    instructions_program: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    associated_token_program: AccountInfo<'info>,
    token_metadata_program: AccountInfo<'info>,
    transfer_params: ExtraTransferParams<'info>,
    signer_seeds: &[&[&[u8]]],
) -> Result<(), Error> {
    let cpi_program = token_metadata_program.to_account_info();
    let cpi_accounts = BridgesplitTransfer {
        authority: authority.to_account_info(),
        payer: payer.to_account_info(),
        token_owner: token_owner.to_account_info(),
        token: from_nft_ta.to_account_info(),
        destination_owner: to.to_account_info(),
        destination: to_nft_ta.to_account_info(),
        mint: nft_mint.to_account_info(),
        metadata: nft_metadata.to_account_info(),
        edition: nft_edition.to_account_info(),
        system_program: system_program.to_account_info(),
        instructions: instructions_program.to_account_info(),
        token_program: token_program.to_account_info(),
        ata_program: associated_token_program.to_account_info(),
    };
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
    bridgesplit_transfer(cpi_ctx, transfer_params, 1)
}

#[allow(clippy::too_many_arguments)]
pub fn delegate_nft<'info>(
    authority: AccountInfo<'info>,
    payer: AccountInfo<'info>,
    mint: AccountInfo<'info>,
    token: AccountInfo<'info>,
    nft_metadata: AccountInfo<'info>,
    delegate: AccountInfo<'info>,
    system_program: AccountInfo<'info>,
    sysvar_instructions: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    token_metadata_program: AccountInfo<'info>,
    signer_seeds: &[&[&[u8]]],
    delegate_params: ExtraDelegateParams<'info>,
) -> Result<(), Error> {
    let cpi_program = token_metadata_program.to_account_info();
    let cpi_accounts = BridgesplitDelegate {
        authority: authority.to_account_info(),
        payer: payer.to_account_info(),
        mint: mint.to_account_info(),
        metadata: nft_metadata.to_account_info(),
        system_program: system_program.to_account_info(),
        token_program: token_program.to_account_info(),
        token: token.to_account_info(),
        delegate: delegate.to_account_info(),
        sysvar_instructions: sysvar_instructions.to_account_info(),
    };
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
    bridgesplit_delegate(cpi_ctx, delegate_params, 1)
}

#[allow(clippy::too_many_arguments)]
pub fn freeze_nft<'info>(
    authority: AccountInfo<'info>,
    payer: AccountInfo<'info>,
    mint: AccountInfo<'info>,
    token: AccountInfo<'info>,
    nft_metadata: AccountInfo<'info>,
    nft_edition: AccountInfo<'info>,
    delegate: AccountInfo<'info>,
    system_program: AccountInfo<'info>,
    sysvar_instructions: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    ata_program: AccountInfo<'info>,
    token_metadata_program: AccountInfo<'info>,
    signer_seeds: &[&[&[u8]]],
    freeze_params: PnftParams<'info>,
) -> Result<(), Error> {
    let cpi_program = token_metadata_program.to_account_info();
    let cpi_accounts = BridgesplitFreeze {
        authority: authority.to_account_info(),
        payer: payer.to_account_info(),
        mint: mint.to_account_info(),
        metadata: nft_metadata.to_account_info(),
        system_program: system_program.to_account_info(),
        token_program: token_program.to_account_info(),
        token: token.to_account_info(),
        token_owner: authority.to_account_info(),
        mpl_token_metadata: token_metadata_program.to_account_info(),
        delegate: delegate.to_account_info(),
        edition: nft_edition.clone(),
        ata_program: ata_program.clone(),
        instructions: sysvar_instructions.to_account_info(),
    };
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
    bridgesplit_freeze(cpi_ctx, freeze_params)
}

#[allow(clippy::too_many_arguments)]
pub fn revoke_nft<'info>(
    authority: AccountInfo<'info>,
    payer: AccountInfo<'info>,
    mint: AccountInfo<'info>,
    token: AccountInfo<'info>,
    delegate: AccountInfo<'info>,
    nft_metadata: AccountInfo<'info>,
    system_program: AccountInfo<'info>,
    sysvar_instructions: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    token_metadata_program: AccountInfo<'info>,
    signer_seeds: &[&[&[u8]]],
    revoke_params: ExtraRevokeParams<'info>,
) -> Result<(), Error> {
    let cpi_program = token_metadata_program.to_account_info();
    let cpi_accounts = BridgesplitRevoke {
        authority: authority.to_account_info(),
        payer: payer.to_account_info(),
        token: token.to_account_info(),
        mint: mint.to_account_info(),
        metadata: nft_metadata.to_account_info(),
        system_program: system_program.to_account_info(),
        token_program: token_program.to_account_info(),
        delegate: delegate.to_account_info(),
        sysvar_instructions: sysvar_instructions.to_account_info(),
    };
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
    bridgesplit_revoke(cpi_ctx, revoke_params)
}

#[allow(clippy::too_many_arguments)]
pub fn unfreeze_nft<'info>(
    authority: AccountInfo<'info>,
    payer: AccountInfo<'info>,
    mint: AccountInfo<'info>,
    token: AccountInfo<'info>,
    delegate: AccountInfo<'info>,
    nft_metadata: AccountInfo<'info>,
    nft_edition: AccountInfo<'info>,
    system_program: AccountInfo<'info>,
    sysvar_instructions: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    associated_token_program: AccountInfo<'info>,
    token_metadata_program: AccountInfo<'info>,
    signer_seeds: &[&[&[u8]]],
    delegate_params: PnftParams<'info>,
) -> Result<(), Error> {
    let cpi_program = token_metadata_program.to_account_info();
    let cpi_accounts = BridgesplitFreeze {
        authority: delegate.to_account_info(),
        payer: payer.to_account_info(),
        token: token.to_account_info(),
        mint: mint.to_account_info(),
        metadata: nft_metadata.to_account_info(),
        system_program: system_program.to_account_info(),
        token_program: token_program.to_account_info(),
        delegate: delegate.to_account_info(),
        token_owner: authority.to_account_info(),
        edition: nft_edition.to_account_info(),
        mpl_token_metadata: token_metadata_program.clone(),
        instructions: sysvar_instructions.clone(),
        ata_program: associated_token_program.clone(),
    };
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
    bridgesplit_thaw(cpi_ctx, delegate_params)
}

fn get_pnft_params(ra: Vec<AccountInfo>) -> PnftParams {
    PnftParams {
        token_record: ra.first().cloned(),
        authorization_rules: ra.get(1).cloned(),
        authorization_rules_program: ra.get(2).cloned(),
        authorization_data: None,
    }
}

/// transfer sol
/// amount in lamports
pub fn transfer_sol<'info>(
    from_account: AccountInfo<'info>,
    to_account: AccountInfo<'info>,
    system_program: AccountInfo<'info>,
    signer_seeds: Option<&[&[&[u8]]; 1]>,
    amount: u64,
) -> Result<(), Error> {
    if let Some(seeds) = signer_seeds {
        invoke_signed(
            &transfer(from_account.key, to_account.key, amount),
            &[
                from_account.to_account_info(),
                to_account.to_account_info(),
                system_program.to_account_info(),
            ],
            seeds,
        )
        .map_err(Into::into)
    } else {
        invoke(
            &transfer(from_account.key, to_account.key, amount),
            &[
                from_account.to_account_info(),
                to_account.to_account_info(),
                system_program.to_account_info(),
            ],
        )
        .map_err(Into::into)
    }
}

/// result of parsing remaining accounts
pub struct ParsedRemainingAccounts<'info> {
    //params for pnft ix's
    pub pnft_params: PnftParams<'info>,
    // delegate record if we're freezing/unfreezing
    pub delegate_record: Option<AccountInfo<'info>>,
    // params for removing existing delegExtraDelegateParams
    pub existing_delegate_params: Option<ExistingDelegateParams<'info>>,
    // apply fee on listings
    pub fees_on: bool,
    pub creator_accounts: Vec<AccountInfo<'info>>,
}

fn parse_pnft_accounts(remaining_accounts: Vec<AccountInfo>) -> PnftParams {
    let account_0 = remaining_accounts.first().unwrap();

    if account_0.key == &Pubkey::default() {
        PnftParams {
            authorization_data: None,
            authorization_rules: None,
            authorization_rules_program: None,
            token_record: None,
        }
    } else {
        get_pnft_params(remaining_accounts)
    }
}

fn parse_existing_delegate_accounts(
    remaining_accounts: Vec<AccountInfo>,
) -> Option<ExistingDelegateParams> {
    let account_0 = remaining_accounts.first().unwrap();

    if account_0.key == &Pubkey::default() {
        None
    } else {
        Some(ExistingDelegateParams {
            existing_delegate: remaining_accounts.first().cloned().unwrap(),
            existing_delegate_record: remaining_accounts.get(1).cloned().unwrap(),
        })
    }
}

fn parse_delegate_record(remaining_accounts: Vec<AccountInfo>) -> Option<AccountInfo> {
    let account_0 = remaining_accounts.first().cloned().unwrap();

    if account_0.key == &Pubkey::default() {
        None
    } else {
        Some(account_0)
    }
}

pub fn parse_remaining_accounts(
    remaining_accounts: Vec<AccountInfo>,
    initializer: Pubkey,
    fees_in_order: bool,
    potential_existing_delegate: bool, //if there is a chance a delegate exista and can interfere
    extra_pnft_accounts: Option<usize>, //if there are extrap nfts tacked onto the end
) -> ParsedRemainingAccounts {
    let mut account_index = 0;
    //first 3 are either default pubkeys or pnft accounts
    let pnft_params = parse_pnft_accounts(remaining_accounts.clone());
    account_index += 3;
    account_index += extra_pnft_accounts.unwrap_or(0);
    let delegate_record = if account_index < remaining_accounts.len() {
        parse_delegate_record(remaining_accounts[account_index..].to_vec())
    } else {
        None
    };
    account_index += 1;
    //next 2 are existing delegate if possible
    let existing_delegate_params =
        if potential_existing_delegate && account_index < remaining_accounts.len() {
            let delegate_accounts =
                parse_existing_delegate_accounts(remaining_accounts[account_index..].to_vec());
            account_index += 2;
            delegate_accounts
        } else {
            None
        };

    account_index += 2;

    let creator_accounts = if account_index < remaining_accounts.len() {
        remaining_accounts[account_index..].to_vec()
    } else {
        Vec::new()
    };

    ParsedRemainingAccounts {
        existing_delegate_params,
        delegate_record,
        pnft_params,
        fees_on,
        creator_accounts,
    }
}

pub fn get_fee_amount(order_price: u64) -> u64 {
    (order_price.checked_mul(PROTOCOL_FEES_BPS))
        .unwrap()
        .checked_div(10000)
        .unwrap()
}

pub fn pay_royalties<'info>(
    price: u64,
    metadata_account: AccountInfo<'info>,
    payer: AccountInfo<'info>,
    system_program: AccountInfo<'info>,
    creator_accounts: Vec<AccountInfo<'info>>,
    use_lamports_transfer: bool,
    signer_seeds: Option<&[&[&[u8]]; 1]>,
) -> Result<(), Error> {
    let metadata = Metadata::safe_deserialize(&metadata_account.data.borrow())?;
    let creator_accounts_map: HashMap<Pubkey, AccountInfo<'info>> = creator_accounts
        .into_iter()
        .map(|creator_account| (*creator_account.key, creator_account))
        .collect();
    let [_, royalties] = get_index_fee_bp(price, metadata.seller_fee_basis_points.into())?;
    if let Some(creators) = metadata.creators.clone() {
        for creator in creators {
            if creator.share != 0 {
                let amount = royalties
                    .checked_mul(creator.share.into())
                    .unwrap()
                    .checked_div(100)
                    .unwrap();
                if use_lamports_transfer {
                    lamport_transfer(
                        payer.clone(),
                        creator_accounts_map
                            .get(&creator.address)
                            .unwrap()
                            .to_account_info(),
                        amount,
                    )?;
                } else {
                    transfer_sol(
                        payer.clone(),
                        creator_accounts_map
                            .get(&creator.address)
                            .unwrap()
                            .to_account_info(),
                        system_program.clone(),
                        signer_seeds,
                        amount,
                    )?;
                }
            }
        }
    }
    Ok(())
}
