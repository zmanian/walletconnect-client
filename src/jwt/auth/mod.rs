mod error;
pub mod token;

pub const RELAY_WEBSOCKET_ADDRESS: &str = "wss://relay.walletconnect.com";
pub(crate) const MULTICODEC_ED25519_BASE: &str = "z";
pub(crate) const MULTICODEC_ED25519_HEADER: [u8; 2] = [237, 1];
pub(crate) const MULTICODEC_ED25519_LENGTH: usize = 32;
pub(crate) const DEFAULT_TOKEN_AUD: &str = RELAY_WEBSOCKET_ADDRESS;
