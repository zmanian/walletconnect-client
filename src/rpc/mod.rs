mod batch;
mod constants;
mod error;
mod fetch;
mod msgid;
mod params;
mod payload;
mod publish;
mod request;
mod response;
mod rpc_response;
mod session;
mod subscribe;
mod subscription;
mod watch;

pub use batch::*;
pub use constants::*;
pub use error::*;
pub use fetch::*;
pub use msgid::*;
pub use params::*;
pub use payload::*;
pub use publish::*;
pub use request::*;
pub use response::*;
pub use rpc_response::*;
pub use session::*;
pub use subscribe::*;
pub use subscription::*;
pub use watch::*;

use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

pub trait Serializable:
    Debug + Clone + PartialEq + Eq + Serialize + DeserializeOwned + Send + Sync + 'static
{
}
impl<T> Serializable for T where
    T: Debug + Clone + PartialEq + Eq + Serialize + DeserializeOwned + Send + Sync + 'static
{
}
