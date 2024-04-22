use crate::jwt::{
    client_id::ClientId,
    decode::{client_id::DecodedClientId, error::ClientIdDecodingError},
};
use derive_more::{AsMut, AsRef};
use ed25519_dalek::VerifyingKey;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, AsRef, AsMut, Serialize, Deserialize)]
#[as_ref(forward)]
#[as_mut(forward)]
pub struct DidKey(
    #[serde(with = "crate::serde_helpers::client_id_as_did_key")] pub DecodedClientId,
);

impl From<DidKey> for VerifyingKey {
    fn from(val: DidKey) -> Self {
        val.0.as_verifying_key()
    }
}

impl From<DecodedClientId> for DidKey {
    fn from(val: DecodedClientId) -> Self {
        Self(val)
    }
}

impl TryFrom<ClientId> for DidKey {
    type Error = ClientIdDecodingError;

    fn try_from(value: ClientId) -> Result<Self, Self::Error> {
        value.decode().map(Self)
    }
}
