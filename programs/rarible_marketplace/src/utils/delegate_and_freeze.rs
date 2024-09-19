use anchor_lang::prelude::*;

use super::{delegate_nft, freeze_nft, metaplex::pnft::utils::PnftParams, DelegateNft, ExtraDelegateParams, FreezeNft};

impl<'info> DelegateAndFreeze<'info> {
    pub fn execute_delegate(
        &self,
        signer_seeds: &[&[&[u8]]],
        delegate_params: ExtraDelegateParams<'info>,
    ) -> Result<()> {
        let accounts = DelegateNft {
            token: self.token.to_account_info(),
            delegate: self.delegate.to_account_info(),
            metadata: self.metadata.to_account_info(),
            mint: self.mint.to_account_info(),
            authority: self.authority.to_account_info(),
            payer: self.payer.to_account_info(),
            token_program: self.token_program.to_account_info(),
            system_program: self.system_program.to_account_info(),
            sysvar_instructions: self.sysvar_instructions.to_account_info(),
        };

        let cpi_ctx = CpiContext::new_with_signer(
            self.mpl_token_metadata_program.to_account_info(),
            accounts,
            signer_seeds,
        );

        delegate_nft(cpi_ctx, delegate_params, 1)
    }

    pub fn execute_freeze_account(
        &self,
        signer_seeds: &[&[&[u8]]],
        freeze_params: PnftParams<'info>,
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

        freeze_nft(cpi_ctx, freeze_params)
    }
}

pub fn delegate_and_freeze<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, DelegateAndFreeze<'info>>,
    signer_seeds: &[&[&[u8]]],
    delegate_params: ExtraDelegateParams<'info>,
    freeze_params: PnftParams<'info>,
) -> Result<()> {
    ctx.accounts
        .execute_delegate(signer_seeds, delegate_params)?;
    ctx.accounts
        .execute_freeze_account(signer_seeds, freeze_params)?;
    Ok(())
}

#[derive(Accounts)]
pub struct DelegateAndFreeze<'info> {
    pub authority: AccountInfo<'info>,
    pub payer: AccountInfo<'info>,
    pub token_owner: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
    pub token: AccountInfo<'info>,
    pub delegate: AccountInfo<'info>,
    pub metadata: AccountInfo<'info>,
    pub edition: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub ata_program: AccountInfo<'info>,
    pub mpl_token_metadata_program: AccountInfo<'info>,
    pub sysvar_instructions: AccountInfo<'info>,
}
