use crate::{
    cipher::{error::CipherError, r#type::Type},
    jwt::decode::{client_id::DecodedClientId, DecodedTopic, Topic},
};
use chacha20poly1305::{
    aead::{
        rand_core::{CryptoRng, RngCore},
        Aead,
    },
    AeadCore, ChaCha20Poly1305, KeyInit, Nonce,
};
use ed25519_dalek::Digest;
use hkdf::Hkdf;
use serde::{de::DeserializeOwned, Serialize};
use sha2::Sha256;
use std::collections::HashMap;
use x25519_dalek::{PublicKey, StaticSecret};

pub mod error;
mod mock;
mod r#type;

pub trait RandProvider: RngCore + CryptoRng + Clone {}

#[derive(Clone)]
pub struct Cipher<R: RandProvider> {
    pub keys: HashMap<Topic, StaticSecret>,
    pub ciphers: HashMap<Topic, ChaCha20Poly1305>,
    rand_provider: R,
}

impl<R: RandProvider> Cipher<R> {
    pub fn new(state: Option<Vec<(Topic, StaticSecret)>>, rand_provider: R) -> Self {
        let mut keys = HashMap::new();
        let mut ciphers = HashMap::new();
        if let Some(state) = state {
            for (topic, key) in state {
                ciphers.insert(topic.clone(), ChaCha20Poly1305::new((&key.to_bytes()).into()));
                keys.insert(topic, key);
            }
        }

        Self { keys, ciphers, rand_provider }
    }

    pub fn generate(&mut self) -> (Topic, StaticSecret) {
        let key = StaticSecret::random_from_rng(&mut self.rand_provider);
        let topic = Topic::generate(&mut self.rand_provider);
        self.register(topic.clone(), key.clone());
        (topic, key)
    }

    pub fn register(&mut self, topic: Topic, key: StaticSecret) {
        self.ciphers.insert(topic.clone(), ChaCha20Poly1305::new((&key.to_bytes()).into()));
        self.keys.insert(topic, key);
    }

    pub fn clear(&mut self) {
        self.ciphers.clear();
        self.keys.clear();
    }

    pub fn encode<T: Serialize>(&self, topic: &Topic, payload: &T) -> Result<String, CipherError> {
        self.encode_with_params(
            topic,
            payload,
            ChaCha20Poly1305::generate_nonce(&mut rand::thread_rng()),
            Type::default(),
        )
    }

    pub fn encode_with_params<T: Serialize>(
        &self,
        topic: &Topic,
        payload: &T,
        nonce: Nonce,
        envelope_type: Type,
    ) -> Result<String, CipherError> {
        let cipher = self.ciphers.get(topic).ok_or(CipherError::UnknownTopic)?;
        let serialized_payload = serde_json::to_string(payload)?;
        let encrypted_payload = cipher.encrypt(&nonce, &*serialized_payload.into_bytes())?;
        let mut envelope = envelope_type.as_bytes();
        envelope.extend(nonce.to_vec());
        envelope.extend(encrypted_payload.to_vec());

        Ok(data_encoding::BASE64.encode(&envelope))
    }

    pub fn decode<T: DeserializeOwned>(
        &self,
        topic: &Topic,
        payload: &str,
    ) -> Result<T, CipherError> {
        let decoded_msg = &self.decode_to_string(topic, payload)?;
        let from_str = serde_json::from_str(decoded_msg);
        Ok(from_str?)
    }

    pub fn create_common_topic(
        &mut self,
        topic: &Topic,
        client_id: DecodedClientId,
    ) -> Result<(Topic, PublicKey), CipherError> {
        let key = self.keys.get(topic).ok_or(CipherError::UnknownTopic)?;
        let static_key = StaticSecret::from(key.to_bytes());
        let public_key = PublicKey::from(client_id.0);

        let (new_topic, expanded_key) = Self::derive_sym_key(static_key, public_key)?;
        self.register(new_topic.clone(), expanded_key.clone());
        Ok((new_topic, PublicKey::from(&expanded_key)))
    }

    pub fn derive_sym_key(
        static_key: StaticSecret,
        public_key: PublicKey,
    ) -> Result<(Topic, StaticSecret), CipherError> {
        let shared_secret = static_key.diffie_hellman(&public_key);

        // let new_key = SigningKey::from_bytes(shared_secret.as_bytes());
        let hk = Hkdf::<Sha256>::new(None, shared_secret.as_ref());
        let mut okm = [0u8; 32];
        hk.expand(&[], &mut okm)?;
        let expanded_key = StaticSecret::from(okm);

        // Ok, got a key. Time for a topic
        let new_topic =
            Topic::from(DecodedTopic::from_bytes(Sha256::digest(expanded_key.as_ref()).into()));

        Ok((new_topic, expanded_key))
    }

    pub fn decode_to_string(&self, topic: &Topic, payload: &str) -> Result<String, CipherError> {
        let encrypted_payload = data_encoding::BASE64.decode(payload.as_bytes())?;

        match Type::from_bytes(&encrypted_payload) {
            Some(Type::Type0) => self.decode_bytes(topic, &encrypted_payload[1..]),
            Some(Type::Type1(_)) => self.decode_bytes(topic, &encrypted_payload[33..]),
            _ => Err(CipherError::CorruptedPayload),
        }
    }

    fn decode_bytes(&self, topic: &Topic, bytes: &[u8]) -> Result<String, CipherError> {
        let cipher = self.ciphers.get(topic).ok_or(CipherError::UnknownTopic)?;
        let decoded_bytes = cipher.decrypt((&bytes[0..12]).into(), &bytes[12..])?;

        Ok(String::from_utf8(decoded_bytes)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cipher::mock::MockRandProvider;
    use ethers::utils::hex;
    use rand::{RngCore, SeedableRng};

    #[test]
    fn test_generate_creates_key_and_topic_based_on_mock() {
        // arrange
        let mut rng = MockRandProvider { next_u32_call: 0, fill_bytes_call: 0 };
        let mut expected_secret = [0u8; 32];
        rng.fill_bytes(&mut expected_secret);
        let mut expected_topic = [0u8; 32];
        for i in 0..32 {
            expected_topic[i] = rng.next_u32() as u8;
        }
        let expected_secret = hex::encode(expected_secret);
        let expected_topic = hex::encode(expected_topic);
        let mut cipher = Cipher::new(None, rng.clone());

        // act
        let (topic, key) = cipher.generate();

        // assert
        assert!(cipher.keys.contains_key(&topic));
        assert!(cipher.ciphers.contains_key(&topic));
        assert_eq!(rng.fill_bytes_call, 1);
        assert_eq!(rng.next_u32_call, 32);

        let topic_value = format!("{}", topic.value());
        let secret_value = hex::encode(key.to_bytes());
        assert_eq!(topic_value, expected_topic);
        assert_eq!(secret_value, expected_secret);
    }

    #[test]
    fn test_generate_unique_keys_and_topics() {
        // arrange
        let mut cipher = Cipher::new(None, rand::rngs::StdRng::from_entropy());
        let mut generated_keys = HashMap::new();
        let mut generated_topic = HashMap::new();

        // act
        for _ in 0..1024 {
            let (topic, key) = cipher.generate();

            // assert
            assert!(cipher.keys.contains_key(&topic));
            assert!(cipher.ciphers.contains_key(&topic));
            assert!(
                !generated_keys.contains_key(&key.clone().to_bytes()),
                "Duplicate key generated"
            );
            assert!(!generated_topic.contains_key(&topic), "Duplicate topic generated");

            generated_topic.insert(topic.clone(), key.clone());
            let key = key.to_bytes();
            generated_keys.insert(key, topic.clone());
        }
    }
}
