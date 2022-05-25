use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Jwt {
    access_token: String,
}

impl Jwt {
    pub fn access_token(&self) -> &str {
        &self.access_token
    }
}

#[derive(serde::Serialize)]
pub struct TokenRequest {
    client_id: String,
    client_secret: String,
    audience: String,
    grant_type: String,
}

impl TokenRequest {
    pub fn new(audience: String) -> Self {
        Self {
            audience,
            client_id: "client_id".to_string(),
            client_secret: "client_secret".to_string(),
            grant_type: "client_credentials".to_string(),
        }
    }
}

#[derive(serde::Serialize)]
pub struct PermissionsForAudience {
    audience: String,
    permissions: Vec<String>,
}

impl PermissionsForAudience {
    pub fn new(audience: String, permissions: Vec<String>) -> Self {
        Self { audience, permissions }
    }
}
