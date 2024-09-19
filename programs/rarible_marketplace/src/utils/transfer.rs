use anchor_lang::prelude::*;
use anchor_spl::token::*;

use mpl_token_metadata::accounts::Metadata;

use crate::utils::metaplex::pnft::{
    transfer::metaplex_transfer,
    utils::{get_is_pnft, AuthorizationDataLocal},
};

#[derive(Accounts)]
pub struct TransferAccounts<'info> {
    pub bs_authority: AccountInfo<'info>,
    pub authority: AccountInfo<'info>,
    pub payer: AccountInfo<'info>,
    pub source: AccountInfo<'info>,
    pub source_ta: AccountInfo<'info>,
    pub destination_owner: AccountInfo<'info>,
    pub destination_ta: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    pub instructions: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub ata_program: AccountInfo<'info>,
    pub rent: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct TransferNft<'info> {
    pub authority: AccountInfo<'info>,
    pub payer: AccountInfo<'info>,
    pub token_owner: AccountInfo<'info>,
    pub token: AccountInfo<'info>,
    pub destination_owner: AccountInfo<'info>,
    pub destination: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
    pub metadata: AccountInfo<'info>,
    pub edition: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    pub instructions: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub ata_program: AccountInfo<'info>,
}

pub struct ExtraTransferParams<'info> {
    pub owner_token_record: Option<AccountInfo<'info>>,
    pub dest_token_record: Option<AccountInfo<'info>>,
    pub authorization_rules: Option<AccountInfo<'info>>,
    pub authorization_data: Option<AuthorizationDataLocal>,
    pub authorization_rules_program: Option<AccountInfo<'info>>,
}

pub fn transfer_nft<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, TransferNft<'info>>,
    params: ExtraTransferParams<'info>,
    amount: u64,
) -> Result<()> {
    let metadata_res = Metadata::safe_deserialize(&ctx.accounts.metadata.data.borrow()[..]);
    let is_pnft = if let Ok(metadata) = metadata_res {
        get_is_pnft(&metadata)
    } else {
        false
    };
    if is_pnft {
        metaplex_transfer(ctx, params, amount)?;
    } else {
        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program,
                Transfer {
                    from: ctx.accounts.token,
                    to: ctx.accounts.destination,
                    authority: ctx.accounts.authority,
                },
                ctx.signer_seeds,
            ),
            amount,
        )?;
    }

    Ok(())
}

#[inline(always)]
pub fn get_extra_transfer_params(
    accounts: Vec<AccountInfo>,
    authorization_data: Option<AuthorizationDataLocal>,
    start_index: usize,
) -> ExtraTransferParams {
    if accounts.len() < start_index + 4 {
        return ExtraTransferParams {
            owner_token_record: None,
            dest_token_record: None,
            authorization_rules: None,
            authorization_data,
            authorization_rules_program: None,
        };
    }

    let owner_token_record: Option<AccountInfo> = accounts.first().cloned();
    let dest_token_record: Option<AccountInfo> = accounts.get(1).cloned();
    let authorization_rules: Option<AccountInfo> = accounts.get(2).cloned();
    let authorization_rules_program: Option<AccountInfo> = accounts.get(3).cloned();

    ExtraTransferParams {
        owner_token_record,
        dest_token_record,
        authorization_rules,
        authorization_data,
        authorization_rules_program,
    }
}

#[derive(Clone)]
pub struct MplTokenMetadata;

impl anchor_lang::Id for MplTokenMetadata {
    fn id() -> Pubkey {
        mpl_token_metadata::ID
    }
}
