use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Claims {
    aud: String,
    iat: Option<i64>,
    exp: Option<i64>,
    scope: String,
    iss: String,
    gty: GrantType,
    #[serde(default)]
    permissions: Vec<String>,
}

impl Claims {
    pub fn new(aud: String, permissions: Vec<String>, iss: String, gty: GrantType) -> Self {
        Self {
            aud,
            iat: Some(chrono::Utc::now().timestamp()),
            exp: Some(chrono::Utc::now().timestamp() + 60000),
            scope: permissions.join(" "),
            iss,
            gty,
            permissions,
        }
    }

    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.iter().any(|x| x == permission)
    }

    pub fn audience(&self) -> &str {
        &self.aud
    }

    pub fn issuer(&self) -> &str {
        &self.iss
    }

    pub fn grant_type(&self) -> &GrantType {
        &self.gty
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum GrantType {
    ClientCredentials,
    AuthorizationCode,
}

impl Display for GrantType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GrantType::ClientCredentials => write!(f, "client_credentials"),
            GrantType::AuthorizationCode => write!(f, "authorization_code"),
        }
    }
}
