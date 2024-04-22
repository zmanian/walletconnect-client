## Quickstart

Add this to your Cargo.toml:

```toml
[dependencies]
walletconnect-client = "0.1"
```

And this to your code:

```rust
use walletconnect_client::prelude::*;
```

To initiate walletconnect connection with the wallet, set up your dApps metadata:

```rust
use walletconnect_client::prelude::*;

let dapp = Metadata::from("Your dApp's name", 
                          "Your dApp's short description", 
                          "https://url.of.your.dapp", 
                          vec!["https://url.to.your.dapps.icon".to_string()]);
```

...and once you'll get your projects id from WalletConnect portal, you can simply create the connection:

```rust 
use walletconnect_client::prelude::*;

const PROJECT_ID: &str = "myprojectidfromwalletconnectportal";

async fn start_session(dapp: Metadata) -> Result<String, WalletConnectError> {
    let client = WalletConnect::connect(PROJECT_ID.into(), 
            1 /* Ethereums chain id */, 
            dapp, 
            None)?;
    let url = client.initiate_session(None).await?;
    Ok(url)
}
```

Now your wallet need to get your sessions url. You can pass it on using url call with proper schema, or present it using qrcode using crates such as `qrcode-generator`:

State loop is manually handled by the implementor (there's no concurrency in some places).
You have to loop somewhere to get any updates from WalletConnect.

```rust
use walletconnect_client::prelude::*;

async fn handle_messages(wc: WalletConnect) {
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
- [ ] Handling non-WASM usage for servers

## Note on WASM

This library currently needs WASM to work. There is a plan to support server-side implementations, though. For now, we focus on building robust solution for WASM implementations of websites.
