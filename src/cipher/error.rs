use thiserror::Error;

#[derive(Debug, Error)]
pub enum CipherError {
    #[error("Unknown topic")]
    UnknownTopic,

    #[error("Encryption error")]
    EncryptionError,

    #[error("Corrupted payload")]
    CorruptedPayload,

    #[error(transparent)]
    CorruptedString(#[from] std::string::FromUtf8Error),

    #[error(transparent)]
    DecodeError(#[from] data_encoding::DecodeError),

    #[error(transparent)]
    CorruptedPacket(#[from] serde_json::error::Error),

    #[error("Invalid key length")]
    InvalidKeyLength,
}

impl From<hkdf::InvalidLength> for CipherError {
    fn from(_: hkdf::InvalidLength) -> Self {
        Self::InvalidKeyLength
    }
}

impl From<chacha20poly1305::Error> for CipherError {
    fn from(_value: chacha20poly1305::Error) -> Self {
        Self::EncryptionError
    }
}
