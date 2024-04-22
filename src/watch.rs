use crate::jwt::decode::Topic;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WatchType {
    Subscriber,
    Publisher,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WatchStatus {
    Accepted,
    Queued,
    Delivered,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WatchAction {
    #[serde(rename = "irn_watchRegister")]
    Register,
    #[serde(rename = "irn_watchUnregister")]
    Unregister,
    #[serde(rename = "irn_watchEvent")]
    WatchEvent,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchEventPayload {
    /// Webhook status. Either `accepted`, `queued` or `delivered`.
    pub status: WatchStatus,
    /// Topic of the message that triggered the watch event.
    pub topic: Topic,
    /// The published message.
    pub message: String,
    /// Message publishing timestamp.
    pub published_at: i64,
    /// Message tag.
    pub tag: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchWebhookPayload {
    /// JWT with [`WatchEventClaims`] payload.
    pub event_auth: String,
}
