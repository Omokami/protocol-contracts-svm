use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_lang::Accounts;
use anchor_lang::{solana_program::account_info::AccountInfo, Key};
use mpl_token_metadata::{
    instructions::MintBuilder,
    types::{AuthorizationData, MintArgs},
};

pub struct MintParams<'info> {
    pub master_edition: Option<AccountInfo<'info>>,
    pub token_record: Option<AccountInfo<'info>>,
    pub delegate_record: Option<AccountInfo<'info>>,
    pub authorization_rules_program: Option<AccountInfo<'info>>,
    pub authorization_rules: Option<AccountInfo<'info>>,
    pub authorization_data: Option<AuthorizationData>,
}

pub fn metaplex_mint<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, MetaplexMint<'info>>,
    params: MintParams<'info>,
) -> Result<()> {
    let mut builder = MintBuilder::new();
    builder
        .token(ctx.accounts.token.key())
        .token_owner(Some(ctx.accounts.token_owner.key()))
        .metadata(ctx.accounts.metadata.key())
        .mint(ctx.accounts.mint.key())
        .authority(ctx.accounts.authority.key())
        .payer(ctx.accounts.payer.key())
        .system_program(ctx.accounts.system_program.key())
        .sysvar_instructions(ctx.accounts.sysvar_instructions.key())
        .spl_token_program(ctx.accounts.spl_token_program.key())
        .spl_ata_program(ctx.accounts.spl_ata_program.key());

    let mut account_infos = vec![
        ctx.accounts.token.to_account_info(),
        ctx.accounts.token_owner.to_account_info(),
        ctx.accounts.metadata.to_account_info(),
    ];

    if let Some(master_edition) = params.master_edition {
        builder.master_edition(Some(master_edition.key()));
        account_infos.push(master_edition.to_account_info());
    }

    if let Some(token_record) = params.token_record {
        builder.token_record(Some(token_record.key()));
        account_infos.push(token_record.to_account_info());
    }

    account_infos.extend(vec![
        ctx.accounts.mint.to_account_info(),
        ctx.accounts.authority.to_account_info(),
    ]);

    if let Some(delegate_record) = params.delegate_record {
        builder.delegate_record(Some(delegate_record.key()));
        account_infos.push(delegate_record.to_account_info());
    }

    account_infos.extend(vec![
        ctx.accounts.payer.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.sysvar_instructions.to_account_info(),
        ctx.accounts.spl_token_program.to_account_info(),
        ctx.accounts.spl_ata_program.to_account_info(),
    ]);

    if let Some(authorization_rules_program) = params.authorization_rules_program {
        builder.authorization_rules_program(Some(authorization_rules_program.key()));
        account_infos.push(authorization_rules_program.to_account_info());
    }

    if let Some(authorization_rules) = params.authorization_rules {
        builder.authorization_rules(Some(authorization_rules.key()));
        account_infos.push(authorization_rules.to_account_info());
    }

    let mint_ix = builder
        .mint_args(MintArgs::V1 {
            amount: 1, // currently only 1
            authorization_data: params.authorization_data,
        })
        .instruction();

    invoke_signed(&mint_ix, &account_infos, ctx.signer_seeds)?;

    Ok(())
}

#[derive(Accounts)]
pub struct MetaplexMint<'info> {
    pub token: AccountInfo<'info>,
    pub token_owner: AccountInfo<'info>,
    pub metadata: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
    pub authority: AccountInfo<'info>,
    pub payer: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    pub sysvar_instructions: AccountInfo<'info>,
    pub spl_token_program: AccountInfo<'info>,
    pub spl_ata_program: AccountInfo<'info>,
}
