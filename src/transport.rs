use async_trait::async_trait;

/// Error type for transport operations.
#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("connection failed: {0}")]
    ConnectionFailed(String),
    #[error("send failed: {0}")]
    SendFailed(String),
    #[error("receive failed: {0}")]
    ReceiveFailed(String),
    #[error("disconnected")]
    Disconnected,
}

/// Trait abstracting WebSocket transport for WalletConnect relay communication.
#[async_trait]
pub trait Transport: Send + Sync {
    /// Connect to the given URL.
    async fn connect(url: &str) -> Result<Self, TransportError>
    where
        Self: Sized;
    /// Send a text message.
    async fn send(&self, msg: String) -> Result<(), TransportError>;
    /// Receive the next text message. Returns None if disconnected.
    async fn recv(&self) -> Result<Option<String>, TransportError>;
}
