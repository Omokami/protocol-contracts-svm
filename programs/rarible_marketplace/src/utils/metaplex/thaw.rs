use anchor_lang::prelude::*;
use anchor_lang::Accounts;
use anchor_lang::{solana_program::account_info::AccountInfo, Key, ToAccountInfos};
use mpl_token_metadata::instructions::ThawDelegatedAccount;

use crate::utils::FreezeNft;

pub fn metaplex_thaw_nft<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, FreezeNft<'info>>,
) -> Result<()> {
    // Instantiate the ThawDelegatedAccount struct
    let thaw_delegated_account = ThawDelegatedAccount {
        delegate: ctx.accounts.delegate.key(),
        token_account: ctx.accounts.token.key(),
        edition: ctx.accounts.edition.key(),
        mint: ctx.accounts.mint.key(),
        token_program: ctx.accounts.token_program.key(),
    };

    // Create the instruction
    let ix = thaw_delegated_account.instruction();

    // Invoke the instruction
    anchor_lang::solana_program::program::invoke_signed(
        &ix,
        &ToAccountInfos::to_account_infos(&ctx),
        ctx.signer_seeds,
    )?;

    Ok(())
}

#[derive(Accounts)]
pub struct MetaplexThawNft<'info> {
    pub delegate: AccountInfo<'info>,
    pub token_account: AccountInfo<'info>,
    pub edition: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
    pub mpl_token_metadata: AccountInfo<'info>,
}
