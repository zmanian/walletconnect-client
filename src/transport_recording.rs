use crate::transport::{Transport, TransportError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEntry {
    /// Monotonic sequence number
    pub seq: u64,
    /// "send" or "recv"
    pub direction: String,
    /// The message content (JSON string)
    pub message: Option<String>,
    /// Timestamp (millis since trace start)
    pub elapsed_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trace {
    /// Identifier for this side of the connection (e.g., "dapp" or "wallet")
    pub role: String,
    pub entries: Vec<TraceEntry>,
}

/// Wraps another Transport, recording all send/recv to a trace.
///
/// Use `new()` to construct — the returned `Arc<Mutex<Trace>>` handle lets you
/// extract the trace after the transport has been moved into `WalletConnect`.
pub struct RecordingTransport<T: Transport> {
    inner: T,
    trace: Arc<Mutex<Trace>>,
    start: std::time::Instant,
    seq: Arc<Mutex<u64>>,
}

impl<T: Transport> RecordingTransport<T> {
    /// Wrap an existing transport for recording.
    ///
    /// Returns `(transport, trace_handle)`. Keep the `trace_handle` to save the
    /// trace after the transport has been consumed by `WalletConnect::new()`.
    pub fn new(inner: T, role: &str) -> (Self, Arc<Mutex<Trace>>) {
        let trace = Arc::new(Mutex::new(Trace {
            role: role.into(),
            entries: vec![],
        }));
        let transport = Self {
            inner,
            trace: Arc::clone(&trace),
            start: std::time::Instant::now(),
            seq: Arc::new(Mutex::new(0)),
        };
        (transport, trace)
    }
}

/// Save a trace to a JSON file.
pub fn save_trace(trace: &Mutex<Trace>, path: &std::path::Path) -> std::io::Result<()> {
    let trace = trace.lock().expect("trace lock poisoned");
    let json = serde_json::to_string_pretty(&*trace)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    std::fs::write(path, json)
}

#[async_trait]
impl<T: Transport> Transport for RecordingTransport<T> {
    async fn connect(url: &str) -> Result<Self, TransportError>
    where
        Self: Sized,
    {
        let _ = url;
        Err(TransportError::ConnectionFailed(
            "use RecordingTransport::new() instead".into(),
        ))
    }

    async fn send(&self, msg: String) -> Result<(), TransportError> {
        let seq = {
            let mut s = self.seq.lock().expect("seq lock poisoned");
            *s += 1;
            *s
        };
        let elapsed = self.start.elapsed().as_millis() as u64;
        {
            let mut trace = self.trace.lock().expect("trace lock poisoned");
            trace.entries.push(TraceEntry {
                seq,
                direction: "send".into(),
                message: Some(msg.clone()),
                elapsed_ms: elapsed,
            });
        }
        self.inner.send(msg).await
    }

    async fn recv(&self) -> Result<Option<String>, TransportError> {
        let result = self.inner.recv().await?;
        let seq = {
            let mut s = self.seq.lock().expect("seq lock poisoned");
            *s += 1;
            *s
        };
        let elapsed = self.start.elapsed().as_millis() as u64;
        {
            let mut trace = self.trace.lock().expect("trace lock poisoned");
            trace.entries.push(TraceEntry {
                seq,
                direction: "recv".into(),
                message: result.clone(),
                elapsed_ms: elapsed,
            });
        }
        Ok(result)
    }
}
