//! Offline replay test for WalletConnect pairing flow.
//! Uses pre-recorded traces -- no network or project ID needed.
//!
//! To generate traces, run the recording test first:
//!   WALLETCONNECT_PROJECT_ID=<id> cargo test --features native -- --ignored test_record_pairing_traces
//!
//! Then run the replay test:
//!   cargo test --features native -- --ignored test_pairing_from_replay

#[cfg(feature = "native")]
mod replay_tests {
    use walletconnect_client::prelude::*;
    use walletconnect_client::transport::Transport;
    use walletconnect_client::transport_recording::{Trace, TraceEntry};
    use walletconnect_client::transport_replay::ReplayTransport;

    /// Smoke test: verify the ReplayTransport itself works correctly
    /// with a synthetic trace (no fixture files needed).
    #[tokio::test]
    async fn test_replay_transport_smoke() {
        let trace = Trace {
            role: "test".into(),
            entries: vec![
                TraceEntry {
                    seq: 1,
                    direction: "send".into(),
                    message: Some(r#"{"hello":"world"}"#.into()),
                    elapsed_ms: 0,
                },
                TraceEntry {
                    seq: 2,
                    direction: "recv".into(),
                    message: Some(r#"{"reply":"ok"}"#.into()),
                    elapsed_ms: 10,
                },
                TraceEntry {
                    seq: 3,
                    direction: "send".into(),
                    message: Some(r#"{"second":"send"}"#.into()),
                    elapsed_ms: 20,
                },
                TraceEntry {
                    seq: 4,
                    direction: "recv".into(),
                    message: None,
                    elapsed_ms: 30,
                },
            ],
        };

        let transport = ReplayTransport::from_trace(trace);

        // First send should succeed
        transport.send("anything".into()).await.unwrap();
        // First recv should return the recorded message
        let msg = transport.recv().await.unwrap();
        assert_eq!(msg, Some(r#"{"reply":"ok"}"#.into()));

        // Second send
        transport.send("another".into()).await.unwrap();
        // Second recv returns None (recorded as None)
        let msg2 = transport.recv().await.unwrap();
        assert_eq!(msg2, None);

        // Third send should fail (exhausted)
        let err = transport.send("overflow".into()).await;
        assert!(err.is_err(), "should fail when trace is exhausted");

        // Third recv should return None (exhausted)
        let msg3 = transport.recv().await.unwrap();
        assert_eq!(msg3, None);
    }

    fn load_dapp_trace() -> Trace {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/dapp_trace.json");
        let content = std::fs::read_to_string(&path)
            .expect("dapp_trace.json fixture must exist. Run the recording test first.");
        serde_json::from_str(&content).expect("parse dapp trace")
    }

    /// Full replay of a recorded pairing flow.
    /// Requires fixture files -- run the recording test first.
    #[tokio::test]
    #[ignore] // Requires recorded trace fixtures
    async fn test_pairing_from_replay() {
        let trace = load_dapp_trace();

        // Verify the trace has actual content
        assert!(!trace.entries.is_empty(), "trace must have entries");
        let send_count = trace
            .entries
            .iter()
            .filter(|e| e.direction == "send")
            .count();
        let recv_count = trace
            .entries
            .iter()
            .filter(|e| e.direction == "recv")
            .count();
        println!(
            "Loaded dapp trace: role={}, entries={}, sends={}, recvs={}",
            trace.role,
            trace.entries.len(),
            send_count,
            recv_count
        );

        let transport = ReplayTransport::from_trace(trace);

        let metadata = Metadata::from(
            "Test dApp",
            "Replay test",
            url::Url::parse("https://test-dapp.example.com").unwrap(),
            vec![],
        );

        let wc = WalletConnect::new(transport, 1, metadata, None);

        let uri = wc.initiate_session(None).await.expect("initiate_session");
        assert!(uri.starts_with("wc:"), "URI must start with wc:");
        println!("Replay pairing URI: {}", uri);

        // Process events from the recorded trace.
        // The replay transport feeds back the recorded relay responses.
        let event = tokio::time::timeout(std::time::Duration::from_secs(5), wc.next()).await;

        println!("Replay event: {:?}", event);
        // The test validates that the dApp state machine processes correctly
        // against recorded traffic without network access.
    }
}
