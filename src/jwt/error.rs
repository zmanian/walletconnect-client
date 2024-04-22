#[derive(Debug, thiserror::Error)]
pub enum JwtError {
    #[error("Invalid format")]
    Format,

    #[error("Invalid encoding")]
    Encoding,

    #[error("Invalid JWT signing algorithm")]
    Header,

    #[error("JWT Token is expired: {:?}", expiration)]
    Expired { expiration: Option<i64> },

    #[error(
        "JWT Token is not yet valid: basic.iat: {}, now + time_leeway: {}, time_leeway: {}",
        basic_iat,
        now_time_leeway,
        time_leeway
    )]
    NotYetValid { basic_iat: i64, now_time_leeway: i64, time_leeway: i64 },

    #[error("Invalid audience")]
    InvalidAudience,

    #[error("Invalid signature")]
    Signature,

    #[error("Encoding keypair mismatch")]
    InvalidKeypair,

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error(transparent)]
    SignatureError(#[from] ethers::core::k256::ecdsa::Error),
}
