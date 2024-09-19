use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke_signed;
use mpl_token_metadata::instructions::RevokeBuilder;

use crate::utils::{RevokeNft, ExtraRevokeParams};

pub fn metaplex_revoke<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, RevokeNft<'info>>,
    params: ExtraRevokeParams<'info>,
) -> Result<()> {
    let mut builder = RevokeBuilder::new();
    builder
        .delegate(ctx.accounts.delegate.key())
        .metadata(ctx.accounts.metadata.key())
        .mint(ctx.accounts.mint.key())
        .authority(ctx.accounts.authority.key())
        .payer(ctx.accounts.payer.key())
        .system_program(ctx.accounts.system_program.key())
        .sysvar_instructions(ctx.accounts.sysvar_instructions.key());

    let mut account_infos = vec![];

    if let Some(delegate_record) = params.delegate_record {
        builder.delegate_record(Some(delegate_record.key()));
        account_infos.push(delegate_record);
    }

    account_infos.extend(vec![
        ctx.accounts.delegate.to_account_info(),
        ctx.accounts.metadata.to_account_info(),
    ]);

    if let Some(master_edition) = params.master_edition {
        builder.master_edition(Some(master_edition.key()));
        account_infos.push(master_edition);
    };

    if let Some(token_record) = params.token_record {
        builder.token_record(Some(token_record.key()));
        account_infos.push(token_record);
    };

    account_infos.push(ctx.accounts.mint.to_account_info());

    builder.token(Some(ctx.accounts.token.key()));
    account_infos.push(ctx.accounts.token.to_account_info());

    account_infos.extend(vec![
        ctx.accounts.authority.to_account_info(),
        ctx.accounts.payer.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.sysvar_instructions.to_account_info(),
    ]);

    builder.spl_token_program(Some(ctx.accounts.token_program.key()));
    account_infos.push(ctx.accounts.token_program.to_account_info());

    if let Some(authorization_rules_program) = params.authorization_rules_program {
        builder.authorization_rules_program(Some(authorization_rules_program.key()));
        account_infos.push(authorization_rules_program);
    };

    if let Some(authorization_rules) = params.authorization_rules {
        builder.authorization_rules(Some(authorization_rules.key()));
        account_infos.push(authorization_rules);
    };

    let revoke_ix = builder.revoke_args(params.revoke_args).instruction();

    invoke_signed(&revoke_ix, &account_infos, ctx.signer_seeds)?;

    Ok(())
}
