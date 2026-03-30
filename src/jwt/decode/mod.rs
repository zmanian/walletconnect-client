use crate::new_type;
use std::sync::Arc;

pub mod client_id;
pub(crate) mod did;
pub mod error;
pub mod sym_key;

use crate::{cipher::RandProvider, jwt::decode::error::DecodingError};
use derive_more::{AsMut, AsRef};
use serde::{Deserialize, Serialize};
use serde_aux::prelude::deserialize_number_from_string;
use std::str::FromStr;
macro_rules! impl_byte_array_newtype {
    ($NewType:ident, $ParentType:ident, $ByteLength:expr) => {
        #[derive(
            Debug, Default, Clone, Hash, PartialEq, Eq, AsRef, AsMut, Serialize, Deserialize,
        )]
        #[as_ref(forward)]
        #[as_mut(forward)]
        #[serde(transparent)]
        pub struct $NewType(pub [u8; $ByteLength]);

        impl $NewType {
            pub const LENGTH: usize = $ByteLength;

            pub fn generate(rand_provider: &mut impl RandProvider) -> Self {
                Self(rand::Rng::gen::<[u8; $ByteLength]>(rand_provider))
            }

            pub fn from_bytes(bytes: [u8; $ByteLength]) -> Self {
                Self(bytes)
            }
        }

        impl FromStr for $NewType {
            type Err = DecodingError;

            fn from_str(val: &str) -> Result<Self, Self::Err> {
                let enc_len = val.len();
                if enc_len == 0 {
                    return Err(DecodingError::Length);
                }

                let dec_len = data_encoding::HEXLOWER_PERMISSIVE
                    .decode_len(enc_len)
                    .map_err(|_| DecodingError::Length)?;

                if dec_len != $ByteLength {
                    return Err(DecodingError::Length);
                }

                let mut data = Self::default();

                data_encoding::HEXLOWER_PERMISSIVE
                    .decode_mut(val.as_bytes(), &mut data.0)
                    .map_err(|_| DecodingError::Encoding)?;

                Ok(data)
            }
        }

        impl std::fmt::Display for $NewType {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&data_encoding::HEXLOWER_PERMISSIVE.encode(&self.0))
            }
        }

        const _: () = {
            impl $ParentType {
                pub fn decode(&self) -> Result<$NewType, DecodingError> {
                    $NewType::try_from(self.clone())
                }

                pub fn generate(rand_provider: &mut impl RandProvider) -> Self {
                    Self::from($NewType::generate(rand_provider))
                }
            }
        };

        impl From<$NewType> for $ParentType {
            fn from(val: $NewType) -> Self {
                Self(val.to_string().into())
            }
        }

        impl TryFrom<$ParentType> for $NewType {
            type Error = DecodingError;

            fn try_from(value: $ParentType) -> Result<Self, Self::Error> {
                value.as_ref().parse()
            }
        }
    };
}

new_type!(
    #[doc = "Represents the topic type."]
    #[as_ref(forward)]
    #[from(forward)]
    Topic: Arc<str>
);

new_type!(
    #[doc = "Represents the subscription ID type."]
    #[as_ref(forward)]
    #[from(forward)]
    SubscriptionId: Arc<str>
);

new_type!(
    #[doc = "Represents the auth token subject type."]
    #[as_ref(forward)]
    #[from(forward)]
    AuthSubject: Arc<str>
);

new_type!(
    #[doc = "Represents the message ID type."]
    #[derive(Copy)]
    MessageId: #[serde(deserialize_with = "deserialize_number_from_string")] u64
);

impl MessageId {
    /// Minimum allowed value of a [`crate::domain::MessageId`].
    const MIN: Self = Self(1000000000);

    pub(crate) fn validate(&self) -> bool {
        self.0 >= Self::MIN.0
    }

    pub fn is_zero(&self) -> bool {
        // Message ID `0` is used when the client request failed to parse for whatever
        // reason, and the server doesn't know the message ID of that request, but still
        // wants to communicate the error.
        self.0 == 0
    }
}

new_type!(
    #[doc = "Represents the project ID type."]
    #[as_ref(forward)]
    #[from(forward)]
    ProjectId: Arc<str>
);

impl_byte_array_newtype!(DecodedTopic, Topic, 32);
impl_byte_array_newtype!(DecodedSubscription, SubscriptionId, 32);
impl_byte_array_newtype!(DecodedAuthSubject, AuthSubject, 32);
impl_byte_array_newtype!(DecodedProjectId, ProjectId, 16);
