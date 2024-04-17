use crate::{
    cipher::{error::CipherError, r#type::Type},
    jwt::decode::{client_id::DecodedClientId, DecodedTopic, Topic},
};
use chacha20poly1305::{aead::Aead, AeadCore, ChaCha20Poly1305, KeyInit, Nonce};
use ed25519_dalek::Digest;
use hkdf::Hkdf;
use serde::{de::DeserializeOwned, Serialize};
use sha2::Sha256;
use std::collections::HashMap;
use x25519_dalek::{PublicKey, StaticSecret};

pub mod error;
mod r#type;

#[derive(Clone)]
pub struct Cipher {
    pub keys: HashMap<Topic, StaticSecret>,
    pub ciphers: HashMap<Topic, ChaCha20Poly1305>,
}

impl Cipher {
    pub fn new(state: Option<Vec<(Topic, StaticSecret)>>) -> Self {
        let mut keys = HashMap::new();
        let mut ciphers = HashMap::new();
        if let Some(state) = state {
            for (topic, key) in state {
                ciphers.insert(topic.clone(), ChaCha20Poly1305::new((&key.to_bytes()).into()));
                keys.insert(topic, key);
            }
        }
        Self { keys, ciphers }
    }

    pub fn generate(&mut self) -> (Topic, StaticSecret) {
        let key = StaticSecret::random_from_rng(&mut rand::thread_rng());
        let topic = Topic::generate();
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
