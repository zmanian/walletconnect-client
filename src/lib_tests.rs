#[cfg(test)]
mod tests {
    use crate::{
        cipher::Cipher,
        jwt::decode::ProjectId,
        metadata::{Metadata, Session},
        ClientState, MessageIdGenerator, State, WalletConnect,
    };
    use futures::{
        channel::{mpsc, mpsc::UnboundedSender},
        Sink, StreamExt,
    };
    use gloo_net::websocket::{Message, WebSocketError};
    use rand::prelude::ThreadRng;
    use regex::Regex;
    use std::{
        collections::HashMap,
        pin::Pin,
        sync::Arc,
        task::{Context, Poll},
    };
    use url::Url;
    use wasm_bindgen::__rt::WasmRefCell;

    // WebSocket's Mock
    struct WebSocketSink<T> {
        inner: UnboundedSender<T>,
    }

    impl<T> Sink<T> for WebSocketSink<T> {
        type Error = WebSocketError;

        fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Pin::new(&mut self.get_mut().inner)
                .poll_ready(cx)
                .map_err(|_| WebSocketError::ConnectionError)
        }

        fn start_send(self: Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
            Pin::new(&mut self.get_mut().inner)
                .start_send(item)
                .map_err(|_| WebSocketError::ConnectionError)
        }

        fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Pin::new(&mut self.get_mut().inner)
                .poll_flush(cx)
                .map_err(|_| WebSocketError::ConnectionError)
        }

        fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Pin::new(&mut self.get_mut().inner)
                .poll_close(cx)
                .map_err(|_| WebSocketError::ConnectionError)
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

        let project_id = ProjectId::from("test_project");
        let chain_id = 1;
        let metadata =
            Metadata::from("test_url", "test_name", Url::parse("ws://local:9722").unwrap(), vec![]);

        let (sink, stream) = mpsc::unbounded::<Message>();

        let stream = stream.map(Ok);
        let sink = WebSocketSink { inner: sink };

        let wallet_connect = WalletConnect {
            sink: Arc::new(WasmRefCell::new(sink)),
            stream: Arc::new(WasmRefCell::new(stream)),
            id_generator: MessageIdGenerator::default(),
            state: Arc::new(WasmRefCell::new(ClientState {
                cipher: Cipher::new(None, ThreadRng::default()),
                subscriptions: HashMap::new(),
                pending: HashMap::new(),
                requests_pending: HashMap::new(),
                state: State::Connecting,
                session: Session::from(metadata, chain_id),
            })),
            chain_id,
        };

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
