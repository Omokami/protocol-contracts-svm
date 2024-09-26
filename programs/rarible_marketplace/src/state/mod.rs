pub const MARKET_SEED: &[u8] = b"market";
pub const ORDER_SEED: &[u8] = b"order";
pub const WALLET_SEED: &[u8] = b"wallet";
pub const VERIFICATION_SEED: &[u8] = b"verification";

pub const PROTOCOL_FEES_BPS: u64 = 50;
pub const PROTOCOL_TREASURY: &str = "";

pub const TOKEN_PID: &str = "";
pub const TOKEN_EXT_PID: &str = "";
pub const BUBBLEGUM_PID: &str = "";

pub const METAPLEX_PID: &str = "";
pub const WNS_PID: &str = "";

pub mod market;
pub mod order;

pub use market::*;
pub use order::*;
