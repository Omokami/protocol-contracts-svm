use anchor_lang::prelude::*;
use anchor_spl::token::*;
use mpl_token_metadata::{accounts::Metadata, types::AuthorizationData};

use crate::utils::{
    metaplex::freeze::metaplex_freeze_nft,
    metaplex::pnft::{
        lock::metaplex_lock,
        utils::{get_is_metaplex_nft, get_is_pnft, PnftParams},
    },
};

#[derive(Accounts)]
pub struct FreezeNft<'info> {
    pub authority: AccountInfo<'info>,
    pub payer: AccountInfo<'info>,
    pub token_owner: AccountInfo<'info>,
    pub token: AccountInfo<'info>,
    pub delegate: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
    pub metadata: AccountInfo<'info>,
    pub edition: AccountInfo<'info>,
    pub mpl_token_metadata: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    pub instructions: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub ata_program: AccountInfo<'info>,
}

impl<'info> FreezeNft<'info> {
    pub fn get_freeze_cpi_ctx(&self) -> CpiContext<'_, '_, '_, 'info, FreezeAccount<'info>> {
        CpiContext::new(
            self.token_program.clone(),
            FreezeAccount {
                account: self.token.clone(),
                mint: self.mint.clone(),
                authority: self.authority.clone(),
            },
        )
    }
}

pub fn freeze_nft<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, FreezeNft<'info>>,
    params: PnftParams<'info>,
) -> Result<()> {
    let is_metaplex_nft = get_is_metaplex_nft(&ctx.accounts.metadata);

    if !is_metaplex_nft {
        //vanilla freeze
        anchor_spl::token::freeze_account(
            ctx.accounts
                .get_freeze_cpi_ctx()
                .with_signer(ctx.signer_seeds),
        )?;
        return Ok(());
    }

    let metadata_res = Metadata::safe_deserialize(&ctx.accounts.metadata.data.borrow()[..]);
    let metadata = metadata_res.unwrap();
    let is_pnft = get_is_pnft(&metadata);
    if is_pnft {
        metaplex_lock(ctx, params)?;
    } else {
        //classic metaplex freeze
        metaplex_freeze_nft(ctx)?;
    }

    Ok(())
}

#[inline(always)]
pub fn get_extra_freeze_params(
    accounts: Vec<AccountInfo>,
    authorization_data: Option<AuthorizationData>,
) -> PnftParams {
    let owner_token_record: Option<AccountInfo> = accounts.first().cloned();
    let authorization_rules: Option<AccountInfo> = accounts.get(1).cloned();
    let authorization_rules_program: Option<AccountInfo> = accounts.get(2).cloned();

    PnftParams {
        token_record: owner_token_record,
        authorization_rules,
        authorization_data,
        authorization_rules_program,
    }
}
