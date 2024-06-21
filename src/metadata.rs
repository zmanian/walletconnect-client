use std::{collections::HashMap, fmt::Display, num::ParseIntError, str::FromStr};

use crate::jwt::decode::Topic;
use chrono::{DateTime, Utc};
use ethers::{types::H160, utils::hex};
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;
use url::Url;

use super::rpc::{SessionParams, SessionPayload};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProtocolOption {
    protocol: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<String>,
}

impl Default for ProtocolOption {
    fn default() -> Self {
        Self { protocol: "irn".to_string(), data: None }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Redirects {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub native: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub universal: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub name: String,
    pub description: String,
    pub url: String,
    pub icons: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verify_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect: Option<Redirects>,
}

impl Metadata {
    pub fn from(name: &str, description: &str, url: Url, icons: Vec<String>) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            url: url.into(),
            icons,
            verify_url: None,
            redirect: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Peer {
    pub public_key: String,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Event {
    #[serde(rename = "chainChanged")]
    ChainChanged,
    #[serde(rename = "accountsChanged")]
    AccountsChanged,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Method {
    #[serde(rename = "personal_sign")]
    Sign,
    #[serde(rename = "eth_signTypedData")]
    SignTypedData,
    #[serde(rename = "eth_signTypedData_v4")]
    SignTypedDataV4,
    #[serde(rename = "eth_signTransaction")]
    SignTransaction,
    #[serde(rename = "eth_sendTransaction")]
    SendTransaction,
}

impl FromStr for Method {
    type Err = bool;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "personal_sign" => Ok(Method::Sign),
            "eth_signTypedData" => Ok(Method::SignTypedData),
            "eth_signTypedData_v4" => Ok(Method::SignTypedDataV4),
            "eth_signTransaction" => Ok(Method::SignTransaction),
            "eth_sendTransaction" => Ok(Method::SendTransaction),
            _ => Err(false),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Chain {
    Eip155(u64),
}

impl From<Chain> for u64 {
    fn from(val: Chain) -> Self {
        match val {
            Chain::Eip155(id) => id,
        }
    }
}

#[derive(Debug, Clone, Error)]
pub enum ChainError {
    #[error("Chain information provided in bad format")]
    BadFormat,

    #[error("Invalid chain type")]
    InvalidType,

    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
}

impl FromStr for Chain {
    type Err = ChainError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let components = s.split(':').collect::<Vec<_>>();
        if components.len() != 2 {
            return Err(ChainError::BadFormat);
        }

        if components[0].to_lowercase() != "eip155" {
            return Err(ChainError::InvalidType);
        }

        Ok(Self::Eip155(components[1].parse::<u64>()?))
    }
}

impl Display for Chain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Eip155(chain_id) => f.write_str(&format!("eip155:{chain_id}")),
        }
    }
}

impl Serialize for Chain {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{self}"))
    }
}
impl<'de> Deserialize<'de> for Chain {
    fn deserialize<D>(deserializer: D) -> Result<Chain, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;

        s.parse::<Chain>().map_err(D::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Namespace {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accounts: Option<Vec<SessionAccount>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chains: Option<Vec<Chain>>,
    pub methods: Vec<Method>,
    pub events: Vec<Event>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub relay: ProtocolOption,
    pub namespaces: Option<HashMap<String, Namespace>>,
    pub required_namespaces: HashMap<String, Namespace>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optional_namespaces: Option<HashMap<String, Namespace>>,
    pub pairing_topic: Option<Topic>,
    pub proposer: Peer,
    pub controller: Option<Peer>,
    pub expiry: Option<DateTime<Utc>>,
    pub chain_id: u64,
}

impl From<Session> for SessionPropose {
    fn from(val: Session) -> Self {
        SessionPropose {
            relays: vec![val.relay],
            required_namespaces: val.required_namespaces,
            optional_namespaces: val.optional_namespaces,
            proposer: val.proposer,
        }
    }
}

impl Session {
    pub fn from(metadata: Metadata, chain_id: u64) -> Self {
        let mut required_namespaces = HashMap::new();
        let mut optional_namespaces = HashMap::new();

        required_namespaces.insert(
            "eip155".to_string(),
            Namespace {
                accounts: None,
                chains: Some(vec![Chain::Eip155(chain_id)]),
                methods: vec![Method::SignTransaction, Method::SignTypedDataV4],
                events: vec![Event::ChainChanged, Event::AccountsChanged],
            },
        );

        optional_namespaces.insert(
            "eip155".to_string(),
            Namespace {
                accounts: None,
                chains: Some(vec![Chain::Eip155(chain_id)]),
                methods: vec![Method::SendTransaction, Method::Sign, Method::SignTypedData],
                events: Vec::new(),
            },
        );

        Self {
            relay: ProtocolOption { protocol: "irn".to_string(), data: None },
            namespaces: None,
            required_namespaces,
            optional_namespaces: Some(optional_namespaces),
            pairing_topic: None,
            proposer: Peer { public_key: "".to_string(), metadata },
            controller: None,
            expiry: None,
            chain_id,
        }
    }

    pub fn settle(&mut self, settlement: &SessionSettlement) {
        self.namespaces = Some(settlement.namespaces.clone());
        self.controller = Some(settlement.controller.clone());
        self.expiry = Some(DateTime::<Utc>::from_timestamp(settlement.expiry, 0).unwrap());
        self.pairing_topic = settlement.pairing_topic.clone();

        self.update_chain_id();
    }

    pub fn update(&mut self, update: &SessionUpdate) {
        self.namespaces = Some(update.namespaces.clone());
        self.update_chain_id();
    }

    pub fn event(&mut self, event: &SessionEvent) {
        match &event.event {
            SessionEventType::AccountsChanged(ref acc_update) => {
                // We replace accounts in namespace

                let new_acc = acc_update.clone();
                if let Some(mut nspaces) = self.namespaces.clone() {
                    if let Some(eip155_namespace) = nspaces.get_mut("eip155") {
                        eip155_namespace.accounts = Some(new_acc.data);
                    }
                }
                // Last but not least - change chain id
                self.chain_id = new_acc.chain_id.into();
            }
            SessionEventType::ChainChanged(ref chain_update) => {
                self.chain_id = chain_update.data;
            }
        }
    }

    pub fn close(&mut self) {
        self.pairing_topic = None;
        self.namespaces = None;
        self.controller = None;
        self.expiry = None;
    }

    pub fn namespace(&self) -> Option<Namespace> {
        if let Some(namespaces) = &self.namespaces {
            if let Some(eip155_namespace) = namespaces.get("eip155") {
                return Some(eip155_namespace.clone());
            }
        }
        None
    }

    pub fn available_networks(&self) -> Vec<u64> {
        let mut chain_ids = Vec::new();
        if let Some(namespace) = self.namespace() {
            if let Some(accounts) = &namespace.accounts {
                for acc in accounts {
                    match acc.chain {
                        Chain::Eip155(chain_id) => {
                            if !chain_ids.contains(&chain_id) {
                                chain_ids.push(chain_id);
                            }
                        }
                    }
                }
            }
        }
        chain_ids
    }

    fn update_chain_id(&mut self) {
        let networks = self.available_networks();
        if !networks.contains(&self.chain_id) {
            self.chain_id = *networks.last().unwrap_or(&0);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRpcRequestData {
    pub method: String,
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRpcRequest {
    pub request: SessionRpcRequestData,
    pub chain_id: Chain,
}

impl SessionRpcRequest {
    pub fn new(method: &str, params: Option<serde_json::Value>, chain_id: u64) -> Self {
        Self {
            request: SessionRpcRequestData { method: method.to_string(), params },
            chain_id: Chain::Eip155(chain_id),
        }
    }
}

impl SessionPayload for SessionRpcRequest {
    fn into_params(self) -> SessionParams {
        SessionParams::Request(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionPropose {
    pub relays: Vec<ProtocolOption>,
    pub required_namespaces: HashMap<String, Namespace>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optional_namespaces: Option<HashMap<String, Namespace>>,
    pub proposer: Peer,
}

impl SessionPayload for SessionPropose {
    fn into_params(self) -> SessionParams {
        SessionParams::Propose(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Responder {
    pub relay: ProtocolOption,
    pub responder_public_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Empty {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSettlement {
    pub relay: ProtocolOption,
    pub namespaces: HashMap<String, Namespace>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_namespaces: Option<HashMap<String, Namespace>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optional_namespaces: Option<HashMap<String, Namespace>>,
    pub pairing_topic: Option<Topic>,
    pub controller: Peer,
    pub expiry: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventAccountsChanged {
    pub data: Vec<SessionAccount>,
    pub chain_id: Chain,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventChainChanged {
    pub data: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionUpdate {
    pub namespaces: HashMap<String, Namespace>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "event")]
pub enum SessionEventType {
    #[serde(rename = "accountsChanged")]
    AccountsChanged(EventAccountsChanged),
    #[serde(rename = "chainChanged")]
    ChainChanged(EventChainChanged),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionEvent {
    pub event: SessionEventType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionDeletion {
    pub message: String,
    pub code: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SessionAccount {
    pub chain: Chain,
    pub account: H160,
}

#[derive(Debug, Clone, Error)]
pub enum SessionAccountError {
    #[error("Account information provided in bad format")]
    BadFormat,

    #[error("Invalid chain type")]
    InvalidType,

    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),

    #[error("Account address in bad format")]
    ParseAccountError,
}

impl FromStr for SessionAccount {
    type Err = SessionAccountError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let components = s.split(':').collect::<Vec<_>>();
        if components.len() != 3 {
            return Err(SessionAccountError::BadFormat);
        }

        if components[0].to_lowercase() != "eip155" {
            return Err(SessionAccountError::InvalidType);
        }

        Ok(SessionAccount {
            chain: Chain::Eip155(components[1].parse::<u64>()?),
            account: H160::from_str(components[2])
                .map_err(|_| SessionAccountError::ParseAccountError)?,
        })
    }
}

impl Display for SessionAccount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{}:{}", self.chain, hex::encode(self.account)))
    }
}

impl Serialize for SessionAccount {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{self}"))
    }
}

impl<'de> Deserialize<'de> for SessionAccount {
    fn deserialize<D>(deserializer: D) -> Result<SessionAccount, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;

        s.parse::<SessionAccount>().map_err(D::Error::custom)
    }
}
