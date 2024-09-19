#![allow(clippy::from_over_into)]
use anchor_lang::prelude::*;
use anchor_spl::token::*;

use borsh::{BorshDeserialize, BorshSerialize};

use mpl_token_metadata::{
    accounts::Metadata,
    types::{DelegateArgs, RevokeArgs, TokenDelegateRole},
};

use super::{metaplex::pnft::{delegate::metaplex_delegate, revoke::metaplex_revoke, utils::{get_delegate, get_is_metaplex_nft, get_is_pnft, ExistingDelegateParams}}, ExtraRevokeParams, FreezeNft, RevokeNft};

#[derive(Accounts)]
pub struct DelegateNft<'info> {
    pub token: AccountInfo<'info>,
    pub delegate: AccountInfo<'info>,
    pub metadata: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
    pub authority: AccountInfo<'info>,
    pub payer: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    pub sysvar_instructions: AccountInfo<'info>,
}

impl<'info> From<FreezeNft<'info>> for DelegateNft<'info> {
    fn from(freeze: FreezeNft) -> DelegateNft {
        DelegateNft {
            token: freeze.token,
            delegate: freeze.delegate,
            metadata: freeze.metadata,
            mint: freeze.mint,
            authority: freeze.authority,
            payer: freeze.payer,
            token_program: freeze.token_program,
            system_program: freeze.system_program,
            sysvar_instructions: freeze.instructions,
        }
    }
}

pub struct ExtraDelegateParams<'info> {
    pub delegate_record: Option<AccountInfo<'info>>,
    pub master_edition: Option<AccountInfo<'info>>,
    pub token_record: Option<AccountInfo<'info>>,
    pub token: Option<AccountInfo<'info>>,
    pub spl_token_program: Option<AccountInfo<'info>>,
    pub authorization_rules_program: Option<AccountInfo<'info>>,
    pub authorization_rules: Option<AccountInfo<'info>>,
    pub existing_delegate_params: Option<ExistingDelegateParams<'info>>,
    pub delegate_args: DelegateArgs,
}

impl<'info> DelegateNft<'info> {
    pub fn get_approve_cpi_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Approve<'info>> {
        CpiContext::new(
            self.token_program.clone(),
            Approve {
                to: self.token.clone(),
                delegate: self.delegate.clone(),
                authority: self.authority.clone(),
            },
        )
    }
}

//copy of mpl to enable conversion from TokenDelegateRole
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum LocalRevokeArgs {
    CollectionV1,
    SaleV1,
    TransferV1,
    UtilityV1,
    StakingV1,
    StandardV1,
    LockedTransferV1,
    ProgrammableConfigV1,
    MigrationV1,
}

impl Into<RevokeArgs> for LocalRevokeArgs {
    fn into(self) -> RevokeArgs {
        match self {
            LocalRevokeArgs::CollectionV1 => RevokeArgs::CollectionV1,
            LocalRevokeArgs::SaleV1 => RevokeArgs::SaleV1,
            LocalRevokeArgs::TransferV1 => RevokeArgs::TransferV1,
            LocalRevokeArgs::UtilityV1 => RevokeArgs::UtilityV1,
            LocalRevokeArgs::StakingV1 => RevokeArgs::StakingV1,
            LocalRevokeArgs::StandardV1 => RevokeArgs::StandardV1,
            LocalRevokeArgs::LockedTransferV1 => RevokeArgs::LockedTransferV1,
            LocalRevokeArgs::ProgrammableConfigV1 => RevokeArgs::ProgrammableConfigV1,
            LocalRevokeArgs::MigrationV1 => RevokeArgs::MigrationV1,
        }
    }
}

impl From<TokenDelegateRole> for LocalRevokeArgs {
    fn from(value: TokenDelegateRole) -> Self {
        match value {
            TokenDelegateRole::Sale => LocalRevokeArgs::SaleV1,
            TokenDelegateRole::Transfer => LocalRevokeArgs::TransferV1,
            TokenDelegateRole::Utility => LocalRevokeArgs::UtilityV1,
            TokenDelegateRole::Staking => LocalRevokeArgs::StakingV1,
            TokenDelegateRole::Standard => LocalRevokeArgs::StandardV1,
            TokenDelegateRole::LockedTransfer => LocalRevokeArgs::LockedTransferV1,
            TokenDelegateRole::Migration => LocalRevokeArgs::MigrationV1,
        }
    }
}

pub fn delegate_nft<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, DelegateNft<'info>>,
    params: ExtraDelegateParams<'info>,
    amount: u64,
) -> Result<()> {
    let is_metaplex_nft = get_is_metaplex_nft(&ctx.accounts.metadata);

    if !is_metaplex_nft {
        //vanilla delegate
        approve(
            ctx.accounts
                .get_approve_cpi_ctx()
                .with_signer(ctx.signer_seeds),
            amount,
        )?;
        return Ok(());
    }

    let metadata_res = Metadata::safe_deserialize(&ctx.accounts.metadata.data.borrow()[..]);
    let metadata = metadata_res.unwrap();
    let is_pnft = get_is_pnft(&metadata);
    if params.token_record.is_some() {
        if let Some(token_delegate) = get_delegate(params.token_record.as_ref().unwrap()) {
            let local_delegate_args: LocalRevokeArgs = LocalRevokeArgs::from(token_delegate.role);
            let revoke_args: RevokeArgs = local_delegate_args.into();

            if let Some(existing_delegate_params) = params.existing_delegate_params.clone() {
                let revoke_ctx = CpiContext::new_with_signer(
                    ctx.program.clone(),
                    RevokeNft {
                        token: ctx.accounts.token.clone(),
                        delegate: existing_delegate_params.existing_delegate.clone(),
                        metadata: ctx.accounts.metadata.clone(),
                        mint: ctx.accounts.mint.clone(),
                        authority: ctx.accounts.authority.clone(),
                        payer: ctx.accounts.payer.clone(),
                        token_program: ctx.accounts.token_program.clone(),
                        system_program: ctx.accounts.system_program.clone(),
                        sysvar_instructions: ctx.accounts.sysvar_instructions.clone(),
                    },
                    <&[&[&[u8]]]>::clone(&ctx.signer_seeds),
                );

                metaplex_revoke(
                    revoke_ctx,
                    ExtraRevokeParams {
                        delegate_record: Some(
                            existing_delegate_params.existing_delegate_record.clone(),
                        ),
                        master_edition: params.master_edition.clone(),
                        token_record: params.token_record.clone(),
                        authorization_rules_program: params.authorization_rules_program.clone(),
                        authorization_rules: params.authorization_rules.clone(),
                        revoke_args,
                    },
                )?;
            }
        }
    }

    if is_pnft {
        metaplex_delegate(ctx, params)?;
    } else {
        //vanilla delegate
        approve(
            ctx.accounts
                .get_approve_cpi_ctx()
                .with_signer(ctx.signer_seeds),
            amount,
        )?;
    }

    Ok(())
}


