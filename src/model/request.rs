use serde::Deserialize;

#[derive(Deserialize)]
pub struct ClientCredentialsTokenRequest {
    pub client_id: String,
    pub client_secret: String,
    pub audience: String,
}

#[derive(Deserialize)]
pub struct AuthorizationCodeTokenRequest {
    pub client_id: String,
    pub client_secret: String,
    pub code: String,
    pub nonce: Option<String>,
    pub redirect_uri: Option<String>,
}

#[derive(Deserialize)]
#[serde(tag = "grant_type", rename_all = "snake_case")]
pub enum TokenRequest {
    AuthorizationCode(AuthorizationCodeTokenRequest),
    ClientCredentials(ClientCredentialsTokenRequest),
}

#[derive(Deserialize)]
pub struct PermissionsForAudienceRequest {
    pub audience: String,
    pub permissions: Vec<String>,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub audience: String,
}
