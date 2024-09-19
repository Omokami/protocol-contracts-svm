use anchor_lang::prelude::*;
use anchor_spl::token::*;
use mpl_token_metadata::{accounts::Metadata, types::RevokeArgs};

use crate::utils::metaplex::pnft::{
    revoke::metaplex_revoke,
    utils::{get_is_metaplex_nft, get_is_pnft},
};
#[derive(Accounts)]
pub struct RevokeNft<'info> {
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

pub struct ExtraRevokeParams<'info> {
    pub delegate_record: Option<AccountInfo<'info>>,
    pub master_edition: Option<AccountInfo<'info>>,
    pub token_record: Option<AccountInfo<'info>>,
    pub authorization_rules_program: Option<AccountInfo<'info>>,
    pub authorization_rules: Option<AccountInfo<'info>>,
    pub revoke_args: RevokeArgs,
}

impl<'info> RevokeNft<'info> {
    pub fn get_revoke_cpi_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Revoke<'info>> {
        CpiContext::new(
            self.token_program.clone(),
            Revoke {
                source: self.token.clone(),
                authority: self.authority.clone(),
            },
        )
    }
}

pub fn revoke_nft<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, RevokeNft<'info>>,
    params: ExtraRevokeParams<'info>,
) -> Result<()> {
    let is_metaplex_nft = get_is_metaplex_nft(&ctx.accounts.metadata);

    if !is_metaplex_nft {
        //vanilla delegate
        revoke(
            ctx.accounts
                .get_revoke_cpi_ctx()
                .with_signer(ctx.signer_seeds),
        )?;
        return Ok(());
    }

    let metadata_res = Metadata::safe_deserialize(&ctx.accounts.metadata.data.borrow()[..]);
    let metadata = metadata_res.unwrap();
    let is_pnft = get_is_pnft(&metadata);
    if is_pnft {
        metaplex_revoke(ctx, params)?;
    } else {
        //vanilla delegate
        revoke(
            ctx.accounts
                .get_revoke_cpi_ctx()
                .with_signer(ctx.signer_seeds),
        )?;
    }

    Ok(())
}

pub fn get_extra_revoke_params(
    accounts: Vec<AccountInfo<'_>>,
    args: RevokeArgs,
) -> ExtraRevokeParams<'_> {
    ExtraRevokeParams {
        delegate_record: accounts.first().cloned(),
        master_edition: accounts.get(1).cloned(),
        token_record: accounts.get(2).cloned(),
        authorization_rules_program: accounts.get(3).cloned(),
        authorization_rules: accounts.get(4).cloned(),
        revoke_args: args,
    }
}
