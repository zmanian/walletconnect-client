use crate::{
    did::{combine_did_data, extract_did_data, DID_METHOD_KEY},
    jwt::{
        auth::{MULTICODEC_ED25519_BASE, MULTICODEC_ED25519_HEADER, MULTICODEC_ED25519_LENGTH},
        decode::{did::DidKey, error::ClientIdDecodingError},
    },
};
use derive_more::{AsMut, AsRef};
use ed25519_dalek::VerifyingKey;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use x25519_dalek::PublicKey;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, AsRef, AsMut, Serialize, Deserialize)]
#[as_ref(forward)]
#[as_mut(forward)]
pub struct DecodedClientId(pub [u8; MULTICODEC_ED25519_LENGTH]);

impl DecodedClientId {
    #[inline]
    pub fn try_from_did_key(did: &str) -> Result<Self, ClientIdDecodingError> {
        extract_did_data(did, DID_METHOD_KEY)?.parse()
    }

    #[inline]
    pub fn to_did_key(&self) -> String {
        combine_did_data(DID_METHOD_KEY, &self.to_string())
    }

    #[inline]
    pub fn from_verifying_key(key: &VerifyingKey) -> Self {
        Self(key.to_bytes())
    }

    #[inline]
    pub fn as_verifying_key(&self) -> VerifyingKey {
        // We know that the length is correct, so we can just unwrap.
        VerifyingKey::from_bytes(&self.0).unwrap()
    }

    #[inline]
    pub fn from_key(key: &PublicKey) -> Self {
        Self(key.to_bytes())
    }

    #[inline]
    pub fn as_public_key(&self) -> PublicKey {
        // We know that the length is correct, so we can just unwrap.
        PublicKey::from(self.0)
    }

    #[inline]
    pub fn to_hex(&self) -> String {
        data_encoding::HEXLOWER_PERMISSIVE.encode(&self.0)
    }

    #[inline]
    pub fn from_hex(string: &str) -> Result<Self, ClientIdDecodingError> {
        Ok(Self((&data_encoding::HEXLOWER_PERMISSIVE.decode(string.as_bytes())?)[..].try_into()?))
    }
}

impl From<VerifyingKey> for DecodedClientId {
    fn from(key: VerifyingKey) -> Self {
        Self::from_verifying_key(&key)
    }
}

impl From<DecodedClientId> for VerifyingKey {
    fn from(val: DecodedClientId) -> Self {
        val.as_verifying_key()
    }
}

impl From<DidKey> for DecodedClientId {
    fn from(val: DidKey) -> Self {
        val.0
    }
}

impl FromStr for DecodedClientId {
    type Err = ClientIdDecodingError;

    fn from_str(val: &str) -> Result<Self, Self::Err> {
        const TOTAL_DECODED_LENGTH: usize =
            MULTICODEC_ED25519_HEADER.len() + MULTICODEC_ED25519_LENGTH;

        let stripped =
            val.strip_prefix(MULTICODEC_ED25519_BASE).ok_or(ClientIdDecodingError::Base)?;

        let mut decoded: [u8; TOTAL_DECODED_LENGTH] = [0; TOTAL_DECODED_LENGTH];

        let decoded_len = bs58::decode(stripped)
            .onto(&mut decoded)
            .map_err(|_| ClientIdDecodingError::Encoding)?;

        if decoded_len != TOTAL_DECODED_LENGTH {
            return Err(ClientIdDecodingError::Length);
        }

        let pub_key = decoded
            .strip_prefix(&MULTICODEC_ED25519_HEADER)
            .ok_or(ClientIdDecodingError::Header)?;

        let mut data = Self::default();
        data.0.copy_from_slice(pub_key);

        Ok(data)
    }
}

impl std::fmt::Display for DecodedClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const PREFIX_LEN: usize = MULTICODEC_ED25519_HEADER.len();
        const TOTAL_LEN: usize = MULTICODEC_ED25519_LENGTH + PREFIX_LEN;

        let mut prefixed_data: [u8; TOTAL_LEN] = [0; TOTAL_LEN];
        prefixed_data[..PREFIX_LEN].copy_from_slice(&MULTICODEC_ED25519_HEADER);
        prefixed_data[PREFIX_LEN..].copy_from_slice(&self.0);

        let encoded_data = bs58::encode(prefixed_data).into_string();

        write!(f, "{MULTICODEC_ED25519_BASE}{encoded_data}")
    }
}
