use anchor_lang::prelude::*;
use num_enum::IntoPrimitive;

pub const MARKET_VERSION: u8 = 1;

#[account()]
pub struct Market {
    /// market account version
    pub version: u8,
    /// mint of the index to which the NFTs belong to
    pub market_identifier: Pubkey,
    /// initializer of the market - can edit and close the market
    pub initializer: Pubkey,
    /// state representing the market - open/closed
    pub state: u8,
    /// reserved space for future changes
    pub reserve: [u8; 512],
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy, PartialEq, IntoPrimitive)]
#[repr(u8)]
pub enum MarketState {
    /// market is open and can be used to create orders
    Open,
    /// market is closed and cannot be used to create orders
    Closed,
}

#[derive(IntoPrimitive)]
#[repr(u8)]
pub enum MarketEditType {
    Init,
}

#[event]
pub struct MarketEditEvent {
    pub edit_type: u8,
    pub address: String,
    pub version: u8,
    pub market_identifier: String,
    pub initializer: String,
    pub state: u8,
}

impl Market {
    /// initialize a new market
    pub fn init(&mut self, market_identifier: Pubkey, initializer: Pubkey) {
        self.version = MARKET_VERSION;
        self.market_identifier = market_identifier;
        self.initializer = initializer;
        self.state = MarketState::Open.into();
    }

    /// return true if the market is active
    pub fn is_active(state: u8) -> bool {
        state != <MarketState as Into<u8>>::into(MarketState::Closed)
    }

    pub fn get_edit_event(
        &mut self,
        address: Pubkey,
        edit_type: MarketEditType,
    ) -> MarketEditEvent {
        MarketEditEvent {
            edit_type: edit_type.into(),
            address: address.to_string(),
            version: self.version,
            market_identifier: self.market_identifier.to_string(),
            initializer: self.initializer.to_string(),
            state: self.state,
        }
    }
}
