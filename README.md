## Quickstart

Add this to your Cargo.toml:

```toml
[dependencies]
walletconnect-client = "0.2"
```

For WASM (browser) usage (default):
```toml
walletconnect-client = { version = "0.2", features = ["wasm"] }
```

For native (server-side) usage:
```toml
walletconnect-client = { version = "0.2", default-features = false, features = ["native"] }
```

And this to your code:

```rust
use walletconnect_client::prelude::*;
```

To initiate walletconnect connection with the wallet, set up your dApps metadata:

```rust
use url::Url;
use walletconnect_client::prelude::*;

let dapp = Metadata::from("Your dApp's name",
                          "Your dApp's short description",
                          Url::parse("https://url.of.your.dapp").expect("Wrong URL"),
                          vec!["https://url.to.your.dapps.icon".to_string()]);
```

...and once you'll get your projects id from WalletConnect portal, you can simply create the connection.

The `WalletConnect` struct is now generic over a `Transport` implementation:

```rust,no_run
use walletconnect_client::prelude::*;

const PROJECT_ID: &str = "myprojectidfromwalletconnectportal";

# #[cfg(feature = "native")]
async fn start_session(dapp: Metadata) -> Result<String, WalletConnectError> {
    let client = WalletConnect::<NativeTransport>::connect(PROJECT_ID.into(),
            1 /* Ethereums chain id */,
            dapp,
            None).await?;
    let url = client.initiate_session(None).await?;
    Ok(url)
}
```

Now your wallet need to get your sessions url. You can pass it on using url call with proper schema, or present it using qrcode using crates such as `qrcode-generator`:

State loop is manually handled by the implementor (there's no concurrency in some places).
You have to loop somewhere to get any updates from WalletConnect.

```rust,no_run
use walletconnect_client::prelude::*;

async fn handle_messages<T: Transport>(wc: WalletConnect<T>) {
    while let Ok(event) = wc.next().await {
        match event {
            Some(event) => println!("Got a new WC event {event:?}"),
            None => println!("This loop brought no new event, and that is fine")
        }
    }
}

```
## Documentation

In progress of creation.

## Features

- [X] Session creation and handling
- [X] Handling transaction signatures
- [X] Handling typed data signatures
- [X] Handling manual chain changes
- [X] Handling events
- [X] Handling pings
- [X] Handling session updates
- [X] Handling session deletion
- [X] Native (tokio-tungstenite) transport support
- [X] WASM (gloo-net) transport support

## Transport

This library supports two transport backends via feature flags:

- `wasm` (default) — Uses `gloo-net` WebSocket for browser/WASM targets
- `native` — Uses `tokio-tungstenite` for server-side / native targets

You can also implement the `Transport` trait for custom WebSocket backends.
