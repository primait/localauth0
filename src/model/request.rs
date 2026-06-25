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
    pub client_secret: Option<String>,
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

/// Query parameters for `GET /authorize`. Mirrors the standard OIDC code-flow
/// authorize request. Unknown extra params (e.g. `prompt`, `acr_values`) are
/// silently ignored by serde — kept lenient because real Auth0 clients send
/// many optional fields we don't simulate.
#[derive(Deserialize, Debug)]
pub struct AuthorizeQuery {
    pub client_id: String,
    pub audience: String,
    pub redirect_uri: String,
    #[serde(default)]
    pub scope: Option<String>,
    /// The caller's `state` — echoed back to `redirect_uri` after a successful login.
    /// Not to be confused with the internal `state_token` localauth0 generates.
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub nonce: Option<String>,
    #[serde(default)]
    pub response_type: Option<String>,
}

/// Shared query shape for every step that threads an internal `state_token`
/// through (`/u/login`, `/u/login/identifier`, `/u/login/password`,
/// `/authorize/resume`).
#[derive(Deserialize, Debug)]
pub struct StateQuery {
    pub state: String,
}

/// Body of `POST /u/login/identifier`. Real Auth0 clients (and the ITF
/// DashboardAutomation) post additional fields like `js-available` and
/// `action: "default"`; serde ignores them.
#[derive(Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct UserLoginIdentifierRequest {
    pub username: String,
}

/// Body of `POST /u/login/password`. Some callers (the ITF DashboardAutomation)
/// also include `state` and `action` in the JSON body — `state` is also in the
/// query string, so the body copy is redundant for us. Extra fields ignored.
#[derive(Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct UserLoginPasswordRequest {
    pub username: String,
    pub password: String,
}
