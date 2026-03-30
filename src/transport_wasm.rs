use crate::transport::{Transport, TransportError};
use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use gloo_net::websocket::{futures::WebSocket, Message};
use std::sync::Mutex;

/// WASM WebSocket transport using gloo-net.
pub struct WasmTransport {
    sink: Mutex<futures::stream::SplitSink<WebSocket, Message>>,
    stream: Mutex<futures::stream::SplitStream<WebSocket>>,
}

#[async_trait]
impl Transport for WasmTransport {
    async fn connect(url: &str) -> Result<Self, TransportError> {
        let ws = WebSocket::open(url).map_err(|e| TransportError::ConnectionFailed(e.to_string()))?;
        let (sink, stream) = ws.split();
        Ok(Self {
            sink: Mutex::new(sink),
            stream: Mutex::new(stream),
        })
    }

    async fn send(&self, msg: String) -> Result<(), TransportError> {
        let mut sink = self.sink.lock().expect("sink lock poisoned");
        sink.send(Message::Text(msg))
            .await
            .map_err(|e| TransportError::SendFailed(e.to_string()))
    }

    async fn recv(&self) -> Result<Option<String>, TransportError> {
        let mut stream = self.stream.lock().expect("stream lock poisoned");
        match stream.next().await {
            Some(Ok(Message::Text(text))) => Ok(Some(text)),
            Some(Ok(Message::Bytes(_))) => {
                Err(TransportError::ReceiveFailed("unexpected binary message".into()))
            }
            Some(Err(e)) => Err(TransportError::ReceiveFailed(e.to_string())),
            None => Ok(None),
        }
    }
}
