use crate::config::CustomField;
use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct ClientCredentialsTokenRequest {
    pub client_id: String,
    pub client_secret: String,
    pub audience: String,
}

#[derive(Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct AuthorizationCodeTokenRequest {
    pub client_id: String,
    pub client_secret: String,
    pub code: String,
    pub nonce: Option<String>,
    pub redirect_uri: Option<String>,
}

#[derive(Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
#[serde(tag = "grant_type", rename_all = "snake_case")]
pub enum TokenRequest {
    AuthorizationCode(AuthorizationCodeTokenRequest),
    ClientCredentials(ClientCredentialsTokenRequest),
}

#[derive(Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct PermissionsForAudienceRequest {
    pub audience: String,
    pub permissions: Vec<String>,
}

#[derive(Deserialize)]
pub struct UpdateCustomClaimsRequest {
    pub custom_claims: Vec<CustomField>,
}

#[derive(Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct UpdateUserInfoRequest {
    pub subject: Option<String>,
    pub name: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub nickname: Option<String>,
    pub locale: Option<String>,
    pub gender: Option<String>,
    pub birthdate: Option<String>,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub picture: Option<String>,
    pub updated_at: Option<DateTime<Utc>>,
    pub custom_fields: Option<Vec<CustomField>>,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub audience: String,
}
