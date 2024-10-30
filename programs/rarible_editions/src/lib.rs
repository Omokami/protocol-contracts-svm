use anchor_lang::prelude::*;

pub mod instructions;
pub use instructions::*;

declare_id!("DsaxZA54w7N9z8jxobtzy3rhQQmXjngjvJNvX1HubtkJ");

pub mod errors;
pub mod state;

pub mod logic;
mod utils;

pub use logic::*;

pub use state::*;

pub mod group_extension_program {
    use anchor_lang::declare_id;
    declare_id!("5hx15GaPPqsYA61v6QpcGPpo125v7rfvEfZQ4dJErG5V");
}

#[program]
pub mod rarible_editions {
    use super::*;

    // v2 endpoints. Prefer these over the original ones. 
    // they allow setting of optional creator co-signer
    // and toggling inscriptions on and off. 
    // for now, creator co-sign is disabled but will be enabled
    // soon to allow for wrapper contracts
    pub fn initialise(ctx: Context<InitialiseCtx>, input: InitialiseInput) -> Result<()> {
        instructions::initialise(ctx, input)
    }

    pub fn mint<'info>(ctx: Context<'_, '_, '_, 'info, MintCtx<'info>>) -> Result<()> {
        instructions::mint(ctx)
    }

    /// add royalties to mint
    pub fn add_royalties(ctx: Context<AddRoyalties>, args: UpdateRoyaltiesArgs) -> Result<()> {
        royalties::add::handler(ctx, args)
    }

    /// modify royalties of mint
    pub fn modify_royalties(
        ctx: Context<ModifyRoyalties>,
        args: UpdateRoyaltiesArgs,
    ) -> Result<()> {
        royalties::modify::handler(ctx, args)
    }

    /// add additional metadata to mint
    pub fn add_metadata(ctx: Context<AddMetadata>, args: Vec<AddMetadataArgs>) -> Result<()> {
        metadata::add::handler(ctx, args)
    }

    /// remove additional metadata to mint
    pub fn remove_metadata(
        ctx: Context<RemoveMetadata>,
        args: Vec<RemoveMetadataArgs>,
    ) -> Result<()> {
        metadata::remove::handler(ctx, args)
    }
}
