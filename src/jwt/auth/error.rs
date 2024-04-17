#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid duration")]
    InvalidDuration,

    #[error("Serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error(transparent)]
    SignatureError(#[from] ethers::core::k256::ecdsa::Error),
}
