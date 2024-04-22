use crate::{
    jwt::claims::{basic::JwtBasicClaims, verifiable::VerifiableClaims},
    watch::{WatchAction, WatchStatus, WatchType},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WatchRegisterClaims {
    /// Basic JWT claims.
    #[serde(flatten)]
    pub basic: JwtBasicClaims,
    /// Action. Must be `irn_watchRegister`.
    pub act: WatchAction,
    /// Watcher type. Either subscriber or publisher.
    pub typ: WatchType,
    /// Webhook URL.
    pub whu: String,
    /// Array of message tags to watch.
    pub tag: Vec<u32>,
    /// Array of statuses to watch.
    pub sts: Vec<WatchStatus>,
}

impl VerifiableClaims for WatchRegisterClaims {
    fn basic(&self) -> &JwtBasicClaims {
        &self.basic
    }
}
