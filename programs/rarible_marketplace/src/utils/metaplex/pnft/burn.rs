use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_lang::Accounts;
use anchor_lang::{solana_program::account_info::AccountInfo, Key};
use mpl_token_metadata::{instructions::BurnBuilder, types::BurnArgs};

pub struct BurnParams<'info> {
    pub collection_metadata: Option<AccountInfo<'info>>,
    pub edition: Option<AccountInfo<'info>>,
    pub master_edition: Option<AccountInfo<'info>>,
    pub master_edition_mint: Option<AccountInfo<'info>>,
    pub master_edition_token: Option<AccountInfo<'info>>,
    pub edition_marker: Option<AccountInfo<'info>>,
    pub token_record: Option<AccountInfo<'info>>,
    pub amount: u64,
}

pub fn metaplex_burn<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, MetaplexBurn<'info>>,
    params: BurnParams<'info>,
) -> Result<()> {
    let mut builder = BurnBuilder::new();
    builder
        .authority(ctx.accounts.authority.key())
        .metadata(ctx.accounts.metadata.key())
        .mint(ctx.accounts.mint.key())
        .token(ctx.accounts.token.key())
        .system_program(ctx.accounts.system_program.key())
        .sysvar_instructions(ctx.accounts.sysvar_instructions.key())
        .spl_token_program(ctx.accounts.spl_token_program.key());

    let mut account_infos = vec![ctx.accounts.metadata.to_account_info()];

    account_infos.extend(vec![ctx.accounts.authority.to_account_info()]);

    if let Some(collection_metadata) = params.collection_metadata {
        builder.collection_metadata(Some(collection_metadata.key()));
        account_infos.push(collection_metadata);
    }

    account_infos.extend(vec![ctx.accounts.metadata.to_account_info()]);

    if let Some(edition) = params.edition {
        builder.edition(Some(edition.key()));
        account_infos.push(edition);
    }

    account_infos.extend(vec![
        ctx.accounts.mint.to_account_info(),
        ctx.accounts.token.to_account_info(),
    ]);

    if let Some(master_edition) = params.master_edition {
        builder.master_edition(Some(master_edition.key()));
        account_infos.push(master_edition);
    }

    if let Some(master_edition_mint) = params.master_edition_mint {
        builder.master_edition_mint(Some(master_edition_mint.key()));
        account_infos.push(master_edition_mint);
    }

    if let Some(master_edition_token) = params.master_edition_token {
        builder.master_edition_token(Some(master_edition_token.key()));
        account_infos.push(master_edition_token);
    }

    if let Some(edition_marker) = params.edition_marker {
        builder.edition_marker(Some(edition_marker.key()));
        account_infos.push(edition_marker);
    }

    if let Some(token_record) = params.token_record {
        builder.token_record(Some(token_record.key()));
        account_infos.push(token_record);
    }

    account_infos.extend(vec![
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.sysvar_instructions.to_account_info(),
        ctx.accounts.spl_token_program.to_account_info(),
    ]);

    let create_ix = builder
        .burn_args(BurnArgs::V1 {
            amount: params.amount,
        })
        .instruction();

    invoke_signed(&create_ix, &account_infos, ctx.signer_seeds)?;

    Ok(())
}

#[derive(Accounts)]
pub struct MetaplexBurn<'info> {
    pub authority: AccountInfo<'info>,
    pub metadata: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
    pub token: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    pub sysvar_instructions: AccountInfo<'info>,
    pub spl_token_program: AccountInfo<'info>,
}
