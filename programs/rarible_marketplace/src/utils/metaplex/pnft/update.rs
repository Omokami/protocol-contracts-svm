use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_lang::Accounts;
use anchor_lang::{solana_program::account_info::AccountInfo, Key};
use mpl_token_metadata::{instructions::UpdateBuilder, types::UpdateArgs};

pub struct UpdateParams<'info> {
    pub update_args: UpdateArgs,
    pub delegate_record: Option<AccountInfo<'info>>,
    pub token: Option<AccountInfo<'info>>,
    pub edition: Option<AccountInfo<'info>>,
    pub authorization_rules_program: Option<AccountInfo<'info>>,
    pub authorization_rules: Option<AccountInfo<'info>>,
}

pub fn metaplex_update<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, MetaplexUpdate<'info>>,
    params: UpdateParams<'info>,
) -> Result<()> {
    let mut builder = UpdateBuilder::new();
    builder
        .authority(ctx.accounts.authority.key())
        .mint(ctx.accounts.mint.key())
        .metadata(ctx.accounts.metadata.key())
        .payer(ctx.accounts.payer.key())
        .system_program(ctx.accounts.system_program.key())
        .sysvar_instructions(ctx.accounts.sysvar_instructions.key());

    let mut account_infos = vec![ctx.accounts.authority.to_account_info()];

    if let Some(delegate_record) = params.delegate_record {
        builder.delegate_record(Some(delegate_record.key()));
        account_infos.push(delegate_record);
    }

    if let Some(token) = params.token {
        builder.token(Some(token.key()));
        account_infos.push(token);
    }

    account_infos.extend(vec![
        ctx.accounts.mint.to_account_info(),
        ctx.accounts.metadata.to_account_info(),
    ]);

    if let Some(edition) = params.edition {
        builder.edition(Some(edition.key()));
        account_infos.push(edition);
    }

    account_infos.extend(vec![
        ctx.accounts.payer.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.sysvar_instructions.to_account_info(),
    ]);

    if let Some(authorization_rules_program) = params.authorization_rules_program {
        builder.authorization_rules_program(Some(authorization_rules_program.key()));
        account_infos.push(authorization_rules_program);
    }

    if let Some(authorization_rules) = params.authorization_rules {
        builder.authorization_rules(Some(authorization_rules.key()));
        account_infos.push(authorization_rules);
    }

    let update_ix = builder.update_args(params.update_args).instruction();

    invoke_signed(&update_ix, &account_infos, ctx.signer_seeds)?;

    Ok(())
}

#[derive(Accounts)]
pub struct MetaplexUpdate<'info> {
    pub authority: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
    pub metadata: AccountInfo<'info>,
    pub payer: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    pub sysvar_instructions: AccountInfo<'info>,
}
