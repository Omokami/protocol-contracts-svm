use anchor_lang::prelude::*;
use num_enum::IntoPrimitive;

pub const WALLET_VERSION: u8 = 1;

#[account()]
/// wallet account - bidding authority and funds holder
pub struct Wallet {
    /// order account version
    pub version: u8,
    /// Owner of the wallet
    pub owner: Pubkey,
    /// wallet balance
    pub balance: u64,
    /// reserved space for future changes
    reserve: [u8; 512],
}

#[derive(IntoPrimitive)]
#[repr(u8)]
pub enum WalletEditType {
    Init,
    Edit,
}

#[event]
pub struct WalletEditEvent {
    pub edit_type: u8,
    pub address: String,
    pub version: u8,
    pub owner: String,
    pub balance: u64,
}

impl Wallet {
    /// initialize a new order account
    pub fn init(&mut self, owner: Pubkey, amount: u64) {
        self.version = WALLET_VERSION;
        self.owner = owner;
        self.balance = amount;
    }

    pub fn edit_balance(&mut self, is_increase: bool, amount: u64) {
        if is_increase {
            self.balance += amount;
        } else {
            self.balance -= amount;
        }
    }

    pub fn get_edit_event(
        &mut self,
        address: Pubkey,
        edit_type: WalletEditType,
    ) -> WalletEditEvent {
        WalletEditEvent {
            edit_type: edit_type.into(),
            address: address.to_string(),
            version: self.version,
            owner: self.owner.to_string(),
            balance: self.balance,
        }
    }
}
