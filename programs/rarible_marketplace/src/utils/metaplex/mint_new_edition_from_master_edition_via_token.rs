use anchor_lang::prelude::*;
use anchor_lang::Accounts;
use anchor_lang::{solana_program::account_info::AccountInfo, Key, ToAccountInfos};

use mpl_token_metadata::{
    instructions::MintNewEditionFromMasterEditionViaToken,
    types::MintNewEditionFromMasterEditionViaTokenArgs,
};

pub fn metaplex_mint_new_edition<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, MetaplexMintNewEdition<'info>>,
    edition: u64,
) -> Result<()> {
    // Create the instruction arguments
    let args =
        mpl_token_metadata::instructions::MintNewEditionFromMasterEditionViaTokenInstructionArgs {
            mint_new_edition_from_master_edition_via_token_args:
                MintNewEditionFromMasterEditionViaTokenArgs { edition },
        };

    // Instantiate the MintNewEditionFromMasterEditionViaToken struct
    let mint_new_edition = MintNewEditionFromMasterEditionViaToken {
        new_metadata: ctx.accounts.new_metadata.key(),
        new_edition: ctx.accounts.new_edition.key(),
        master_edition: ctx.accounts.master_edition.key(),
        new_mint: ctx.accounts.new_mint.key(),
        edition_mark_pda: ctx.accounts.edition_mark_pda.key(),
        new_mint_authority: ctx.accounts.manager_account.key(),
        payer: ctx.accounts.payer_account.key(),
        token_account_owner: ctx.accounts.manager_account.key(), // Assuming manager_account is the owner; adjust if necessary
        token_account: ctx.accounts.token_account.key(),
        new_metadata_update_authority: ctx.accounts.manager_account.key(), // Assuming manager_account is the authority; adjust if necessary
        metadata: ctx.accounts.metadata.key(),
        token_program: ctx.accounts.token_program.key(),
        system_program: ctx.accounts.system.key(),
        rent: Some(ctx.accounts.rent.key()),
    };

    // Generate the instruction
    let ix = mint_new_edition.instruction_with_remaining_accounts(args, &[]);

    // Invoke the instruction
    anchor_lang::solana_program::program::invoke_signed(
        &ix,
        &ToAccountInfos::to_account_infos(&ctx),
        ctx.signer_seeds,
    )?;

    Ok(())
}

#[derive(Accounts)]
pub struct MetaplexMintNewEdition<'info> {
    pub new_metadata: AccountInfo<'info>,
    pub new_edition: AccountInfo<'info>,
    pub master_edition: AccountInfo<'info>,
    pub new_mint: AccountInfo<'info>,
    pub manager_account: AccountInfo<'info>,
    pub payer_account: AccountInfo<'info>,
    pub token_account: AccountInfo<'info>,
    pub metadata: AccountInfo<'info>,
    pub metadata_mint: AccountInfo<'info>,
    pub edition_mark_pda: AccountInfo<'info>,
    pub mpl_token_metadata: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub system: AccountInfo<'info>,
    pub rent: AccountInfo<'info>,
}
