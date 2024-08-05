pub use crate::app_data::*;
pub use claims::*;
pub use id_token::*;
pub use jwks::*;
pub use openid_metadata::*;
pub use request::*;
pub use response::*;
pub use user_info::*;

pub mod certificates;
mod claims;
pub mod defaults;
mod id_token;
mod jwks;
mod openid_metadata;
mod request;
mod response;
mod user_info;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(transparent)]
pub struct Issuer(pub String);

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(transparent)]
pub struct Subject(pub String);
