use anchor_lang::prelude::*;
use anchor_lang::Accounts;
use anchor_lang::{solana_program::account_info::AccountInfo, Key};
use mpl_token_metadata::{
    instructions::{UpdateMetadataAccountV2, UpdateMetadataAccountV2InstructionArgs},
    types::{Collection, Creator, DataV2},
};

use anchor_lang::solana_program::pubkey::Pubkey;

pub fn metaplex_update_metadata_account<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, MetaplexUpdateMetadataAccount<'info>>,
    name: String,
    symbol: String,
    uri: String,
    creators: Option<Vec<Creator>>,
    seller_fee_basis_points: u16,
    collection: Option<Collection>,
) -> Result<()> {
    // Prepare the instruction arguments
    let args = UpdateMetadataAccountV2InstructionArgs {
        data: Some(DataV2 {
            name,
            symbol,
            uri,
            seller_fee_basis_points,
            creators,
            collection,
            uses: None, // Assuming 'uses' is None, adjust as necessary
        }),
        new_update_authority: None, // Assuming no change to update authority, adjust as necessary
        primary_sale_happened: None, // Assuming no change, adjust as necessary
        is_mutable: None,           // Assuming no change to mutability, adjust as necessary
    };

    // Instantiate the UpdateMetadataAccountV2 struct
    let update_metadata_account_v2 = UpdateMetadataAccountV2 {
        metadata: ctx.accounts.metadata_account.key(),
        update_authority: ctx.accounts.manager_account.key(),
    };

    // Create the instruction
    let ix = update_metadata_account_v2.instruction_with_remaining_accounts(args, &[]);

    // Invoke the instruction
    anchor_lang::solana_program::program::invoke_signed(
        &ix,
        &[
            ctx.accounts.metadata_account.to_account_info(),
            ctx.accounts.manager_account.to_account_info(),
            // Include other necessary accounts here
        ],
        ctx.signer_seeds,
    )?;

    Ok(())
}
#[derive(Accounts)]
pub struct MetaplexUpdateMetadataAccount<'info> {
    pub metadata_account: AccountInfo<'info>,
    pub manager_account: AccountInfo<'info>,
    pub mpl_token_metadata: AccountInfo<'info>,
}
