use crate::jwt::{claims::verifiable::VerifiableClaims, decode::did::DidKey};
use serde::{Deserialize, Serialize};

/// Basic JWT claims that are common to all JWTs used by the Relay.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JwtBasicClaims {
    /// Client ID matching the watch type.
    pub iss: DidKey,
    /// Relay URL.
    pub aud: String,
    /// Service URL.
    pub sub: String,
    /// Issued at, timestamp.
    pub iat: i64,
    /// Expiration, timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exp: Option<i64>,
}

impl VerifiableClaims for JwtBasicClaims {
    fn basic(&self) -> &JwtBasicClaims {
        self
    }
}
