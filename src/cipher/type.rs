use ed25519_dalek::VerifyingKey;

#[derive(Debug, Clone, Copy, Default)]
pub enum Type {
    #[default]
    Type0,
    Type1(VerifyingKey),
}

impl Type {
    pub(crate) fn as_bytes(&self) -> Vec<u8> {
        match self {
            Type::Type1(key) => {
                let mut envelope = vec![1u8];
                envelope.extend(key.as_bytes().to_vec());
                envelope
            }
            _ => vec![0u8],
        }
    }

    pub(crate) fn from_bytes(bytes: &[u8]) -> Option<Self> {
        match bytes[0] {
            0u8 => Some(Self::Type0),
            1u8 => match VerifyingKey::from_bytes((&bytes[1..32]).try_into().unwrap()) {
                Ok(key) => Some(Self::Type1(key)),
                _ => None,
            },
            _ => None,
        }
    }
}
