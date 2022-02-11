use serde::Deserialize;

#[derive(Deserialize)]
pub struct TokenRequest {
    pub client_id: String,
    pub client_secret: String,
    pub audience: String,
    pub grant_type: String,
}

#[derive(Deserialize)]
pub struct PermissionsForAudienceRequest {
    pub audience: String,
    pub permissions: Vec<String>,
}
