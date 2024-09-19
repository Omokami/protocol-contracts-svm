use anchor_lang::prelude::*;

use super::{metaplex::pnft::utils::PnftParams, revoke_nft, thaw_nft, ExtraRevokeParams, FreezeNft, RevokeNft};

impl<'info> ThawAndRevoke<'info> {
    pub fn execute_thaw_nft(
        &self,
        signer_seeds: &[&[&[u8]]],
        thaw_params: PnftParams<'info>,
    ) -> Result<()> {
        let accounts = FreezeNft {
            authority: self.authority.to_account_info(),
            payer: self.payer.to_account_info(),
            token_owner: self.token_owner.to_account_info(),
            token: self.token.to_account_info(),
            delegate: self.delegate.to_account_info(),
            mint: self.mint.to_account_info(),
            metadata: self.metadata.to_account_info(),
            edition: self.edition.to_account_info(),
            mpl_token_metadata: self.mpl_token_metadata_program.to_account_info(),
            system_program: self.system_program.to_account_info(),
            instructions: self.sysvar_instructions.to_account_info(),
            token_program: self.token_program.to_account_info(),
            ata_program: self.ata_program.to_account_info(),
        };

        let cpi_ctx = CpiContext::new_with_signer(
            self.mpl_token_metadata_program.to_account_info(),
            accounts,
            signer_seeds,
        );

        thaw_nft(cpi_ctx, thaw_params)
    }

    pub fn execute_revoke_nft(
        &self,
        signer_seeds: &[&[&[u8]]],
        revoke_params: ExtraRevokeParams<'info>,
    ) -> Result<()> {
        let accounts = RevokeNft {
            authority: self.token_owner.to_account_info(),
            payer: self.payer.to_account_info(),
            token: self.token.to_account_info(),
            delegate: self.delegate.to_account_info(),
            mint: self.mint.to_account_info(),
            metadata: self.metadata.to_account_info(),
            system_program: self.system_program.to_account_info(),
            sysvar_instructions: self.sysvar_instructions.to_account_info(),
            token_program: self.token_program.to_account_info(),
        };

        let cpi_ctx = CpiContext::new_with_signer(
            self.mpl_token_metadata_program.to_account_info(),
            accounts,
            signer_seeds,
        );

        revoke_nft(cpi_ctx, revoke_params)
    }
}

pub fn thaw_and_revoke<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, ThawAndRevoke<'info>>,
    signer_seeds: &[&[&[u8]]],
    revoke: bool,
    thaw_params: PnftParams<'info>,
    revoke_params: ExtraRevokeParams<'info>,
) -> Result<()> {
    ctx.accounts
        .execute_thaw_nft(signer_seeds, thaw_params)?;
    if revoke {
        ctx.accounts
            .execute_revoke_nft(signer_seeds, revoke_params)?;
    }
    Ok(())
}

#[derive(Accounts)]
pub struct ThawAndRevoke<'info> {
    pub authority: AccountInfo<'info>,
    pub payer: AccountInfo<'info>,
    pub token_owner: AccountInfo<'info>,
    pub token: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
    pub metadata: AccountInfo<'info>,
    pub edition: AccountInfo<'info>,
    pub delegate: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub ata_program: AccountInfo<'info>,
    pub mpl_token_metadata_program: AccountInfo<'info>,
    pub sysvar_instructions: AccountInfo<'info>,
}
