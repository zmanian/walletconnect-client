use crate::transport::{Transport, TransportError};
use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite::Message};

type WsStream =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

/// Native WebSocket transport using tokio-tungstenite.
pub struct NativeTransport {
    sink: Mutex<futures::stream::SplitSink<WsStream, Message>>,
    stream: Mutex<futures::stream::SplitStream<WsStream>>,
}

#[async_trait(?Send)]
impl Transport for NativeTransport {
    async fn connect(url: &str) -> Result<Self, TransportError> {
        let (ws_stream, _) = connect_async(url)
            .await
            .map_err(|e| TransportError::ConnectionFailed(e.to_string()))?;
        let (sink, stream) = ws_stream.split();
        Ok(Self {
            sink: Mutex::new(sink),
            stream: Mutex::new(stream),
        })
    }

    async fn send(&self, msg: String) -> Result<(), TransportError> {
        let mut sink = self.sink.lock().await;
        sink.send(Message::Text(msg.into()))
            .await
            .map_err(|e| TransportError::SendFailed(e.to_string()))
    }

    async fn recv(&self) -> Result<Option<String>, TransportError> {
        let mut stream = self.stream.lock().await;
        match stream.next().await {
            Some(Ok(Message::Text(text))) => Ok(Some(text.to_string())),
            Some(Ok(Message::Binary(_))) => {
                Err(TransportError::ReceiveFailed("unexpected binary message".into()))
            }
            Some(Ok(Message::Close(_))) => Ok(None),
            Some(Ok(_)) => {
                // Ping, Pong, Frame — skip and recurse would be ideal but
                // we keep it simple: treat as no message
                Ok(None)
            }
            Some(Err(e)) => Err(TransportError::ReceiveFailed(e.to_string())),
            None => Ok(None),
        }
    }
}
