use anchor_lang::prelude::*;
use anchor_lang::Accounts;
use anchor_lang::{solana_program::account_info::AccountInfo, Key, ToAccountInfos};
use mpl_token_metadata::instructions::{
    CreateMasterEditionV3, CreateMasterEditionV3InstructionArgs,
};

pub fn metaplex_create_master_edition<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, MetaplexCreateMasterEdition<'info>>,
    max_supply: Option<u64>, // Assuming max_supply can be None
) -> Result<()> {
    let accounts = CreateMasterEditionV3 {
        edition: ctx.accounts.edition.key(),
        mint: ctx.accounts.mint.key(),
        update_authority: ctx.accounts.manager_account.key(),
        mint_authority: ctx.accounts.manager_account.key(),
        payer: ctx.accounts.initializer_account.key(),
        metadata: ctx.accounts.metadata_account.key(),
        token_program: ctx.accounts.token_program.key(),
        system_program: ctx.accounts.system.key(),
        rent: Some(ctx.accounts.rent.to_account_info().key()),
    };

    let args = CreateMasterEditionV3InstructionArgs {
        max_supply, // Adjust this based on the actual structure
    };

    let ix = accounts.instruction(args);

    anchor_lang::solana_program::program::invoke_signed(
        &ix,
        &ToAccountInfos::to_account_infos(&ctx),
        ctx.signer_seeds,
    )?;

    Ok(())
}
#[derive(Accounts)]
pub struct MetaplexCreateMasterEdition<'info> {
    pub edition: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
    pub manager_account: AccountInfo<'info>,
    pub metadata_account: AccountInfo<'info>,
    pub initializer_account: AccountInfo<'info>,
    pub mpl_token_metadata: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub system: AccountInfo<'info>,
    pub rent: AccountInfo<'info>,
}
