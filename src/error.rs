use crate::{cipher::error::CipherError, jwt::decode::error::ClientIdDecodingError};
use ethers::prelude::{JsonRpcError, ProviderError, RpcError};
use gloo_net::websocket::WebSocketError;

#[derive(Debug, thiserror::Error)]
/// WalletConnect error.
pub enum Error {
    #[error("Query error")]
    Query,

    #[error("Url error")]
    Url,

    #[error("Token error")]
    Token,

    #[error("Disconnected")]
    Disconnected,

    #[error("BadParameter")]
    BadParam,

    #[error("Unknown error")]
    Unknown,

    #[error("Bad response")]
    BadResponse,

    #[error("Wallet error")]
    WalletError(JsonRpcError),

    #[error(transparent)]
    ClientIdDecodingError(#[from] ClientIdDecodingError),

    #[error(transparent)]
    CipherError(#[from] CipherError),

    #[error(transparent)]
    CorruptedPacket(#[from] serde_json::error::Error),

    #[error(transparent)]
    WebSocketError(#[from] WebSocketError),

    #[error(transparent)]
    JSError(#[from] gloo_utils::errors::JsError),
}

impl RpcError for Error {
    fn as_error_response(&self) -> Option<&JsonRpcError> {
        match self {
            Error::WalletError(e) => Some(e),
            _ => None,
        }
    }

    fn is_error_response(&self) -> bool {
        self.as_error_response().is_some()
    }

    fn as_serde_error(&self) -> Option<&serde_json::Error> {
        match self {
            Error::CorruptedPacket(e) => Some(e),
            _ => None,
        }
    }

    fn is_serde_error(&self) -> bool {
        self.as_serde_error().is_some()
    }
}

impl From<Error> for ProviderError {
    fn from(src: Error) -> Self {
        ProviderError::JsonRpcClientError(Box::new(src))
    }
}
