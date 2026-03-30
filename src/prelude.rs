pub use super::{
    error::Error as WalletConnectError,
    event::Event,
    metadata::Metadata,
    transport::Transport,
    WalletConnect,
};

#[cfg(feature = "wasm")]
pub use super::transport_wasm::WasmTransport;

#[cfg(feature = "native")]
pub use super::transport_native::NativeTransport;
