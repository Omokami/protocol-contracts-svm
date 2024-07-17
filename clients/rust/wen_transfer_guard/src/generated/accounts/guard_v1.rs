//! This code was AUTOGENERATED using the kinobi library.
//! Please DO NOT EDIT THIS FILE, instead use visitors
//! to add features, then rerun kinobi to update it.
//!
//! <https://github.com/kinobi-so/kinobi>
//!

use crate::generated::types::CpiRule;
use crate::generated::types::MetadataAdditionalFieldRule;
use crate::generated::types::TransferAmountRule;
use borsh::BorshDeserialize;
use borsh::BorshSerialize;
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GuardV1 {
    pub discriminator: [u8; 8],
    /// Mint token representing the guard, do not confuse with the mint of the token being transferred.
    #[cfg_attr(
        feature = "serde",
        serde(with = "serde_with::As::<serde_with::DisplayFromStr>")
    )]
    pub mint: Pubkey,
    /// Bump seed for the guard account.
    pub bump: u8,
    /// CPI ruleset for the guard.
    pub cpi_rule: Option<CpiRule>,
    /// Transfer amount ruleset for the guard.
    pub transfer_amount_rule: Option<TransferAmountRule>,
    /// Additional fields ruleset for the guard.
    pub additional_fields_rule: Vec<MetadataAdditionalFieldRule>,
}

impl GuardV1 {
    #[inline(always)]
    pub fn from_bytes(data: &[u8]) -> Result<Self, std::io::Error> {
        let mut data = data;
        Self::deserialize(&mut data)
    }
}

impl<'a> TryFrom<&solana_program::account_info::AccountInfo<'a>> for GuardV1 {
    type Error = std::io::Error;

    fn try_from(
        account_info: &solana_program::account_info::AccountInfo<'a>,
    ) -> Result<Self, Self::Error> {
        let mut data: &[u8] = &(*account_info.data).borrow();
        Self::deserialize(&mut data)
    }
}

#[cfg(feature = "anchor")]
impl anchor_lang::AccountDeserialize for GuardV1 {
    fn try_deserialize_unchecked(buf: &mut &[u8]) -> anchor_lang::Result<Self> {
        Ok(Self::deserialize(buf)?)
    }
}

#[cfg(feature = "anchor")]
impl anchor_lang::AccountSerialize for GuardV1 {}

#[cfg(feature = "anchor")]
impl anchor_lang::Owner for GuardV1 {
    fn owner() -> Pubkey {
        crate::WEN_TRANSFER_GUARD_ID
    }
}

#[cfg(feature = "anchor-idl-build")]
impl anchor_lang::IdlBuild for GuardV1 {}

#[cfg(feature = "anchor-idl-build")]
impl anchor_lang::Discriminator for GuardV1 {
    const DISCRIMINATOR: [u8; 8] = [0; 8];
}
