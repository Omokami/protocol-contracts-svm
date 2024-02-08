#![allow(ambiguous_glob_reexports)]

use anchor_lang::prelude::*;

pub mod errors;
pub mod instructions;
pub mod state;
pub mod utils;

pub use errors::*;
pub use instructions::*;
pub use state::*;
pub use utils::*;

declare_id!("8e9NZefQowF1ViN4eiz8r3wgKw9xLESGkkQEZJWox49o");

#[program]
pub mod wen_new_standard {
    use super::*;

    /*
        Token group instructions
    */
    /// create group
    pub fn create_group_account(
        ctx: Context<CreateGroupAccount>,
        args: CreateGroupAccountArgs,
    ) -> Result<()> {
        instructions::group::create::handler(ctx, args)
    }

    /*
        Token mint instructions
    */
    /// create mint
    pub fn create_mint_account(
        ctx: Context<CreateMintAccount>,
        args: CreateMintAccountArgs,
    ) -> Result<()> {
        instructions::mint::create::handler(ctx, args)
    }

    /// add mint to group
    pub fn add_group_to_mint(ctx: Context<AddGroup>) -> Result<()> {
        instructions::mint::group::add::handler(ctx)
    }
}
