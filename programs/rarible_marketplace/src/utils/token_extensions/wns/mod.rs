use anchor_lang::prelude::*;

pub struct WnsApprovalAccounts<'info> {
    pub approval_account: AccountInfo<'info>,
    pub distribution_account: AccountInfo<'info>,
    pub distribution_token_account: AccountInfo<'info>,
    pub distribution_program: AccountInfo<'info>,
    pub payment_mint: AccountInfo<'info>,
}
