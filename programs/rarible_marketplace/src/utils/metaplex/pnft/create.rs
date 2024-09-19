use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_lang::Accounts;
use anchor_lang::{solana_program::account_info::AccountInfo, Key};
use mpl_token_metadata::{
    instructions::CreateBuilder,
    types::{Collection, CollectionDetails, CreateArgs, Creator, PrintSupply, TokenStandard, Uses},
};

pub struct CreateParams<'info> {
    pub master_edition: Option<AccountInfo<'info>>,
    pub initialize_mint: bool,
    pub update_authority_as_signer: bool,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub seller_fee_basis_points: u16,
    pub creators: Option<Vec<Creator>>,
    pub primary_sale_happened: bool,
    pub is_mutable: bool,
    pub token_standard: TokenStandard,
    pub collection: Option<Collection>,
    pub uses: Option<Uses>,
    pub collection_details: Option<CollectionDetails>,
    pub rule_set: Option<Pubkey>,
    pub decimals: Option<u8>,
    pub print_supply: Option<PrintSupply>,
}

pub fn metaplex_create<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, MetaplexCreate<'info>>,
    params: CreateParams<'info>,
) -> Result<()> {
    let mut builder = CreateBuilder::new();
    builder
        .metadata(ctx.accounts.metadata.key())
        .mint(ctx.accounts.mint.key(), params.initialize_mint)
        .authority(ctx.accounts.authority.key())
        .payer(ctx.accounts.payer.key())
        .update_authority(
            ctx.accounts.update_authority.key(),
            params.update_authority_as_signer,
        )
        .system_program(ctx.accounts.system_program.key())
        .sysvar_instructions(ctx.accounts.sysvar_instructions.key())
        .spl_token_program(Some(ctx.accounts.spl_token_program.key()));

    let mut account_infos = vec![ctx.accounts.metadata.to_account_info()];

    if let Some(master_edition) = params.master_edition {
        builder.master_edition(Some(master_edition.key()));
        account_infos.push(master_edition);
    }

    account_infos.extend(vec![
        ctx.accounts.mint.to_account_info(),
        ctx.accounts.authority.to_account_info(),
        ctx.accounts.payer.to_account_info(),
        ctx.accounts.update_authority.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.sysvar_instructions.to_account_info(),
        ctx.accounts.spl_token_program.to_account_info(),
    ]);

    let create_ix = builder
        .create_args(CreateArgs::V1 {
            name: params.name,
            symbol: params.symbol,
            uri: params.uri,
            seller_fee_basis_points: params.seller_fee_basis_points,
            creators: params.creators,
            primary_sale_happened: params.primary_sale_happened,
            is_mutable: params.is_mutable,
            token_standard: params.token_standard,
            collection: params.collection,
            uses: params.uses,
            collection_details: params.collection_details,
            rule_set: params.rule_set,
            decimals: params.decimals,
            print_supply: params.print_supply,
        })
        .instruction();

    invoke_signed(&create_ix, &account_infos, ctx.signer_seeds)?;

    Ok(())
}

#[derive(Accounts)]
pub struct MetaplexCreate<'info> {
    pub metadata: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
    pub authority: AccountInfo<'info>,
    pub payer: AccountInfo<'info>,
    pub update_authority: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    pub sysvar_instructions: AccountInfo<'info>,
    pub spl_token_program: AccountInfo<'info>,
}
