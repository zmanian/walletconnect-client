use crate::{
    jwt::decode::{client_id::DecodedClientId, did::DidKey, error::ClientIdDecodingError},
    new_type,
};
use std::sync::Arc;
new_type!(
    #[doc = "Represents the client ID type."]
    #[as_ref(forward)]
    #[from(forward)]
    ClientId: Arc<str>
);

impl ClientId {
    pub fn decode(&self) -> Result<DecodedClientId, ClientIdDecodingError> {
        DecodedClientId::try_from(self.clone())
    }
}

impl From<DecodedClientId> for ClientId {
    fn from(val: DecodedClientId) -> Self {
        Self(val.to_string().into())
    }
}

impl TryFrom<ClientId> for DecodedClientId {
    type Error = ClientIdDecodingError;

    fn try_from(value: ClientId) -> Result<Self, Self::Error> {
        value.as_ref().parse()
    }
}

impl From<DidKey> for ClientId {
    fn from(val: DidKey) -> Self {
        val.0.into()
    }
}
