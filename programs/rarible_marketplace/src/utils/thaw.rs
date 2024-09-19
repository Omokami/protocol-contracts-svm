use anchor_lang::prelude::*;
use mpl_token_metadata::accounts::Metadata;

use crate::utils::{
    metaplex::{thaw::metaplex_thaw_nft,
    pnft::{
        unlock::metaplex_unlock,
        utils::{get_is_metaplex_nft, get_is_pnft, PnftParams},
    }},
    FreezeNft,
};

pub fn thaw_nft<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, FreezeNft<'info>>,
    params: PnftParams<'info>,
) -> Result<()> {
    let is_metaplex_nft = get_is_metaplex_nft(&ctx.accounts.metadata);

    if !is_metaplex_nft {
        //vanilla unfreeze
        return Ok(());
    }

    let metadata_res = Metadata::safe_deserialize(&ctx.accounts.metadata.data.borrow()[..]);
    let metadata = metadata_res.unwrap();
    let is_pnft = get_is_pnft(&metadata);
    if is_pnft {
        metaplex_unlock(ctx, params)?;
    } else {
        // classic metaplex unfreeze
        metaplex_thaw_nft(ctx)?;
    }

    Ok(())
}
