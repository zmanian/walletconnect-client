use crate::jwt::{
    auth::{MULTICODEC_ED25519_BASE, MULTICODEC_ED25519_HEADER, MULTICODEC_ED25519_LENGTH},
    decode::error::ClientIdDecodingError,
};
use derive_more::{AsMut, AsRef};
use ed25519_dalek::SecretKey;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, AsRef, AsMut, Serialize, Deserialize)]
#[as_ref(forward)]
#[as_mut(forward)]
pub struct DecodedSymKey(pub [u8; MULTICODEC_ED25519_LENGTH]);

impl DecodedSymKey {
    #[inline]
    pub fn from_key(key: &SecretKey) -> Self {
        Self(*key)
    }

    #[inline]
    pub fn as_secret_key(&self) -> SecretKey {
        // We know that the length is correct, so we can just unwrap.
        self.0
    }
}

impl std::fmt::Display for DecodedSymKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&data_encoding::HEXLOWER_PERMISSIVE.encode(&self.0))
    }
}

impl FromStr for DecodedSymKey {
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

        let sym_key = decoded
            .strip_prefix(&MULTICODEC_ED25519_HEADER)
            .ok_or(ClientIdDecodingError::Header)?;

        let mut data = Self::default();
        data.0.copy_from_slice(sym_key);

        Ok(data)
    }
}
