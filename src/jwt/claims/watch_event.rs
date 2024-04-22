use crate::{
    jwt::claims::{basic::JwtBasicClaims, verifiable::VerifiableClaims},
    watch::{WatchAction, WatchEventPayload, WatchType},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WatchEventClaims {
    /// Basic JWT claims.
    #[serde(flatten)]
    pub basic: JwtBasicClaims,
    /// Action. Must be `irn_watchEvent`.
    pub act: WatchAction,
    /// Watcher type. Either subscriber or publisher.
    pub typ: WatchType,
    /// Webhook URL.
    pub whu: String,
    /// Event payload.
    pub evt: WatchEventPayload,
}

impl VerifiableClaims for WatchEventClaims {
    fn basic(&self) -> &JwtBasicClaims {
        &self.basic
    }
}
