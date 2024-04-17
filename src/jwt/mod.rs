mod auth;
mod claims;
mod client_id;
pub(crate) mod decode;
pub mod error;
pub mod header;

pub use auth::{
    token::{AuthToken, SerializedAuthToken},
    RELAY_WEBSOCKET_ADDRESS,
};

const JWT_DELIMITER: &str = ".";
const JWT_HEADER_TYP: &str = "JWT";
const JWT_HEADER_ALG: &str = "EdDSA";
const JWT_VALIDATION_TIME_LEEWAY_SECS: i64 = 120;
