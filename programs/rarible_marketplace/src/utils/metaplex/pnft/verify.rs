use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_lang::Accounts;
use anchor_lang::{solana_program::account_info::AccountInfo, Key};
use mpl_token_metadata::{instructions::VerifyBuilder, types::VerificationArgs};

pub struct VerifyParams<'info> {
    pub delegate_record: Option<AccountInfo<'info>>,
    pub collection_mint: Option<AccountInfo<'info>>,
    pub collection_metadata: Option<AccountInfo<'info>>,
    pub collection_master_edition: Option<AccountInfo<'info>>,
    pub verify_type: VerificationArgs,
}

pub fn metaplex_verify<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, MetaplexVerify<'info>>,
    params: VerifyParams<'info>,
) -> Result<()> {
    let mut builder = VerifyBuilder::new();
    builder
        .authority(ctx.accounts.authority.key())
        .metadata(ctx.accounts.metadata.key())
        .system_program(ctx.accounts.system_program.key())
        .sysvar_instructions(ctx.accounts.sysvar_instructions.key());

    let mut account_infos = vec![ctx.accounts.authority.to_account_info()];

    if let Some(delegate_record) = params.delegate_record {
        builder.delegate_record(Some(delegate_record.key()));
        account_infos.push(delegate_record);
    }

    account_infos.push(ctx.accounts.metadata.to_account_info());

    if let Some(collection_mint) = params.collection_mint {
        builder.collection_mint(Some(collection_mint.key()));
        account_infos.push(collection_mint);
    }

    if let Some(collection_metadata) = params.collection_metadata {
        builder.collection_metadata(Some(collection_metadata.key()));
        account_infos.push(collection_metadata);
    }

    if let Some(collection_master_edition) = params.collection_master_edition {
        builder.collection_master_edition(Some(collection_master_edition.key()));
        account_infos.push(collection_master_edition);
    }

    account_infos.extend(vec![
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.sysvar_instructions.to_account_info(),
    ]);

    let verify_ix = builder.verification_args(params.verify_type).instruction();

    invoke_signed(&verify_ix, &account_infos, ctx.signer_seeds)?;

    Ok(())
}

#[derive(Accounts)]
pub struct MetaplexVerify<'info> {
    pub authority: AccountInfo<'info>,
    pub metadata: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    pub sysvar_instructions: AccountInfo<'info>,
}
