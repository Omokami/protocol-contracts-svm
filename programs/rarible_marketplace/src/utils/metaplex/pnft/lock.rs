use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_lang::Accounts;
use anchor_lang::{solana_program::account_info::AccountInfo, Key};
use mpl_token_metadata::{
    accounts::Metadata,
    instructions::LockBuilder,
    types::{LockArgs, ProgrammableConfig, TokenStandard},
};

use crate::utils::{FreezeNft, metaplex::pnft::utils::PnftParams};

pub fn metaplex_lock<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, FreezeNft<'info>>,
    params: PnftParams<'info>,
) -> Result<()> {
    let mut builder = LockBuilder::new();
    builder
        .authority(ctx.accounts.delegate.key())
        .token_owner(Some(ctx.accounts.token_owner.key()))
        .token(ctx.accounts.token.key())
        .mint(ctx.accounts.mint.key())
        .metadata(ctx.accounts.metadata.key())
        .edition(Some(ctx.accounts.edition.key()))
        .payer(ctx.accounts.payer.key())
        .system_program(ctx.accounts.system_program.key())
        .sysvar_instructions(ctx.accounts.instructions.key())
        .spl_token_program(Some(ctx.accounts.token_program.key()));

    let mut account_infos = vec![
        ctx.accounts.delegate.to_account_info(),
        ctx.accounts.token_owner.to_account_info(),
        ctx.accounts.token.to_account_info(),
        ctx.accounts.mint.to_account_info(),
        ctx.accounts.metadata.to_account_info(),
        ctx.accounts.edition.to_account_info(),
        ctx.accounts.payer.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.instructions.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
    ];

    let metadata = Metadata::safe_deserialize(&ctx.accounts.metadata.data.borrow()[..])?;
    if let Some(standard) = metadata.token_standard {
        msg!("standard triggered");
        if standard == TokenStandard::ProgrammableNonFungible {
            builder.token_record(Some(params.token_record.clone().unwrap().key()));
            account_infos.push(params.token_record.unwrap().to_account_info());
        }
    }

    //if auth rules passed in, validate & include it in CPI call
    if let Some(config) = metadata.programmable_config {
        match config {
            ProgrammableConfig::V1 { rule_set } => {
                if let Some(_rule_set) = rule_set {
                    msg!("ruleset triggered");
                    //safe to unwrap here, it's expected
                    let authorization_rules = params.authorization_rules.unwrap();
                    let rules_program = params.authorization_rules_program.unwrap();

                    //2. add to builder
                    builder.authorization_rules(Some(*authorization_rules.key));
                    builder.authorization_rules_program(Some(*rules_program.key));

                    //3. add to accounts
                    account_infos.push(authorization_rules.to_account_info());
                    account_infos.push(rules_program.to_account_info());
                }
            }
        }
    }

    let lock_ix = builder
        .lock_args(LockArgs::V1 {
            authorization_data: params.authorization_data,
        })
        .instruction();

    invoke_signed(&lock_ix, &account_infos, ctx.signer_seeds)?;

    Ok(())
}

#[derive(Accounts)]
pub struct MetaplexLock<'info> {
    pub authority: AccountInfo<'info>,
    pub payer: AccountInfo<'info>,
    pub token_owner: AccountInfo<'info>,
    pub token: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
    pub metadata: AccountInfo<'info>,
    pub edition: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    pub instructions: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
}
