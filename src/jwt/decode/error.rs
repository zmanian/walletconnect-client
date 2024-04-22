use crate::did::DidError;
use data_encoding::DecodeError;
use std::{array::TryFromSliceError, convert::Infallible};

#[derive(Debug, Clone, thiserror::Error)]
pub enum ClientIdDecodingError {
    #[error("Invalid issuer multicodec base")]
    Base,

    #[error("Invalid issuer base58")]
    Encoding,

    #[error("Invalid multicodec header")]
    Header,

    #[error("Invalid DID key data: {0}")]
    Did(#[from] DidError),

    #[error("Invalid issuer pubkey length")]
    Length,

    #[error(transparent)]
    DecodingError(#[from] DecodingError),

    #[error(transparent)]
    DecodeError(#[from] DecodeError),

    #[error(transparent)]
    Infallible(#[from] Infallible),

    #[error(transparent)]
    TryFromSliceError(#[from] TryFromSliceError),
}

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum DecodingError {
    #[error("Invalid encoding")]
    Encoding,

    #[error("Invalid data length")]
    Length,
}
