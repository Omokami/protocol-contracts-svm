use anchor_lang::prelude::*;
use anchor_lang::Accounts;
use anchor_lang::{solana_program::account_info::AccountInfo, Key, ToAccountInfos};
use mpl_token_metadata::types::DataV2;
use mpl_token_metadata::{
    instructions::{CreateMetadataAccountV3, CreateMetadataAccountV3InstructionArgs},
    types::{Collection, Creator},
};

#[inline(never)]
pub fn metaplex_create_metadata_account<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, MetaplexCreateMetadataAccount<'info>>,
    name: String,
    symbol: String,
    uri: String,
    creators: Option<Vec<Creator>>,
    seller_fee_basis_points: u16,
    collection: Option<Collection>,
) -> Result<()> {
    // Construct the instruction arguments
    let args = CreateMetadataAccountV3InstructionArgs {
        data: DataV2 {
            name,
            symbol,
            uri,
            creators,
            seller_fee_basis_points,
            collection,
            uses: None,
        },
        is_mutable: true,
        collection_details: None,
    };

    // Create the CreateMetadataAccountV3 struct
    let create_metadata_account_v3 = CreateMetadataAccountV3 {
        metadata: ctx.accounts.metadata_account.key(),
        mint: ctx.accounts.mint.key(),
        mint_authority: ctx.accounts.manager_account.key(),
        payer: ctx.accounts.initializer_account.key(),
        update_authority: (ctx.accounts.manager_account.key(), true), // Assuming the update authority is the manager account and is a signer
        system_program: ctx.accounts.system.key(),
        rent: Some(ctx.accounts.rent.key()), // Assuming rent is not optional in your context
    };

    // Generate the instruction
    let ix = create_metadata_account_v3.instruction_with_remaining_accounts(args, &[]);

    // Invoke the instruction
    anchor_lang::solana_program::program::invoke_signed(
        &ix,
        &ToAccountInfos::to_account_infos(&ctx),
        ctx.signer_seeds,
    )?;

    Ok(())
}
#[derive(Accounts)]
pub struct MetaplexCreateMetadataAccount<'info> {
    pub metadata_account: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
    pub initializer_account: AccountInfo<'info>,
    pub manager_account: AccountInfo<'info>,
    pub mpl_token_metadata: AccountInfo<'info>,
    pub system: AccountInfo<'info>,
    pub rent: AccountInfo<'info>,
}
