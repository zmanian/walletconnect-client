use crate::transport::{Transport, TransportError};
use crate::transport_recording::Trace;
use async_trait::async_trait;
use std::sync::Mutex;

/// Replays a recorded trace without any network access.
pub struct ReplayTransport {
    trace: Trace,
    /// Current position in send entries
    send_pos: Mutex<usize>,
    /// Current position in recv entries
    recv_pos: Mutex<usize>,
}

impl ReplayTransport {
    /// Load a replay transport from a JSON trace file.
    pub fn from_file(path: &std::path::Path) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let trace: Trace = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(Self::from_trace(trace))
    }

    /// Create a replay transport from an in-memory trace.
    pub fn from_trace(trace: Trace) -> Self {
        Self {
            trace,
            send_pos: Mutex::new(0),
            recv_pos: Mutex::new(0),
        }
    }
}

#[async_trait]
impl Transport for ReplayTransport {
    async fn connect(_url: &str) -> Result<Self, TransportError>
    where
        Self: Sized,
    {
        Err(TransportError::ConnectionFailed(
            "use ReplayTransport::from_file() or from_trace() instead".into(),
        ))
    }

    async fn send(&self, _msg: String) -> Result<(), TransportError> {
        let mut pos = self.send_pos.lock().expect("send_pos lock poisoned");
        let sends: Vec<_> = self
            .trace
            .entries
            .iter()
            .filter(|e| e.direction == "send")
            .collect();
        if *pos < sends.len() {
            *pos += 1;
            Ok(())
        } else {
            Err(TransportError::SendFailed(
                "replay trace exhausted (no more send entries)".into(),
            ))
        }
    }

    async fn recv(&self) -> Result<Option<String>, TransportError> {
        let mut pos = self.recv_pos.lock().expect("recv_pos lock poisoned");
        let recvs: Vec<_> = self
            .trace
            .entries
            .iter()
            .filter(|e| e.direction == "recv")
            .collect();
        if *pos < recvs.len() {
            let entry = recvs[*pos];
            *pos += 1;
            Ok(entry.message.clone())
        } else {
            // Trace exhausted — signal disconnection
            Ok(None)
        }
    }
}
