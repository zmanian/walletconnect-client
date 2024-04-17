use crate::{
    jwt::claims::{basic::JwtBasicClaims, verifiable::VerifiableClaims},
    watch::{WatchAction, WatchType},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WatchUnregisterClaims {
    /// Basic JWT claims.
    #[serde(flatten)]
    pub basic: JwtBasicClaims,
    /// Action. Must be `irn_watchUnregister`.
    pub act: WatchAction,
    /// Watcher type. Either subscriber or publisher.
    pub typ: WatchType,
    /// Webhook URL.
    pub whu: String,
}

impl VerifiableClaims for WatchUnregisterClaims {
    fn basic(&self) -> &JwtBasicClaims {
        &self.basic
    }
}
