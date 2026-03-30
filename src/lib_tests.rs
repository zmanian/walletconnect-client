#[cfg(test)]
mod tests {
    use crate::{
        cipher::Cipher,
        jwt::decode::ProjectId,
        metadata::{Metadata, Session},
        transport::{Transport, TransportError},
        ClientState, MessageIdGenerator, State, WalletConnect,
    };
    use async_trait::async_trait;
    use futures::channel::mpsc;
    use futures::StreamExt;
    use rand::prelude::ThreadRng;
    use regex::Regex;
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };
    use url::Url;

    /// Mock transport for testing.
    struct MockTransport {
        sender: mpsc::UnboundedSender<String>,
        receiver: Mutex<mpsc::UnboundedReceiver<String>>,
    }

    #[async_trait(?Send)]
    impl Transport for MockTransport {
        async fn connect(_url: &str) -> Result<Self, TransportError> {
            Err(TransportError::ConnectionFailed("mock".into()))
        }

        async fn send(&self, msg: String) -> Result<(), TransportError> {
            self.sender
                .unbounded_send(msg)
                .map_err(|e| TransportError::SendFailed(e.to_string()))
        }

        async fn recv(&self) -> Result<Option<String>, TransportError> {
            let mut rx = self.receiver.lock().expect("receiver lock poisoned");
            match rx.next().await {
                Some(msg) => Ok(Some(msg)),
                None => Ok(None),
            }
        }
    }

    impl MockTransport {
        fn new() -> (Self, mpsc::UnboundedSender<String>, mpsc::UnboundedReceiver<String>) {
            let (outgoing_tx, outgoing_rx) = mpsc::unbounded::<String>();
            let (incoming_tx, incoming_rx) = mpsc::unbounded::<String>();
            (
                Self {
                    sender: outgoing_tx,
                    receiver: Mutex::new(incoming_rx),
                },
                incoming_tx,
                outgoing_rx,
            )
        }
    }

    #[tokio::test]
    async fn test_wallet_connect_session_initialization() {
        // arrange
        const EXPECTED_PROTOCOL: &str = "wc";
        const EXPECTED_VERSION: &str = "2";
        const RELAY_PROTOCOL_PARAMETER_NAME: &str = "relay-protocol";
        const RELAY_PROTOCOL_PARAMETER_VALUE: &str = "irn";

        const SYM_KEY_PARAMETER_NAME: &str = "symKey";

        const KEY_LENGTH: usize = 64;

        let chain_id = 1;
        let metadata =
            Metadata::from("test_url", "test_name", Url::parse("ws://local:9722").unwrap(), vec![]);

        let (transport, _incoming_tx, _outgoing_rx) = MockTransport::new();

        let wallet_connect = WalletConnect::new(transport, chain_id, metadata, None);

        // act
        let result = wallet_connect.initiate_session(None).await;

        // assert
        assert!(result.is_ok());
        let result = result.unwrap();
        let re =
            Regex::new(r"(wc):([a-fA-F0-9]*)@(2)\?(relay-protocol)=(irn)&(symKey)=([a-fA-F0-9]*)")
                .unwrap();
        let (protocol, topic, version, relay_protocol, irn, sym_key, sym_key_value) =
            match re.captures(result.as_str()).map(|cap| {
                let protocol = cap[1].to_string();
                let topic = cap[2].to_string();
                let version = cap[3].to_string();
                let relay_protocol = cap[4].to_string();
                let irn = cap[5].to_string();
                let sym_key = cap[6].to_string();
                let sym_key_value = cap[7].to_string();
                (protocol, topic, version, relay_protocol, irn, sym_key, sym_key_value)
            }) {
                Some((protocol, topic, version, relay_protocol, irn, sym_key, sym_key_value)) => {
                    (protocol, topic, version, relay_protocol, irn, sym_key, sym_key_value)
                }
                None => {
                    panic!("No match found");
                }
            };

        assert_eq!(protocol, EXPECTED_PROTOCOL);
        assert_eq!(topic.len(), KEY_LENGTH);
        assert!(topic.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(!topic.chars().all(|c| c == '0'));
        assert_eq!(version, EXPECTED_VERSION);
        assert_eq!(relay_protocol, RELAY_PROTOCOL_PARAMETER_NAME);
        assert_eq!(irn, RELAY_PROTOCOL_PARAMETER_VALUE);
        assert_eq!(sym_key, SYM_KEY_PARAMETER_NAME);
        assert_eq!(sym_key_value.len(), KEY_LENGTH);
        assert!(sym_key_value.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(!sym_key_value.chars().all(|c| c == '0'))
    }
}
