use crate::error::Error;
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};

use crate::model::Jwk;

#[derive(Serialize, Deserialize, Debug)]
pub struct Claims {
    aud: String,
    exp: Option<i64>,
    #[serde(default)]
    permissions: Vec<String>,
}

impl Claims {
    pub fn new(aud: String, permissions: Vec<String>) -> Self {
        let exp: Option<i64> = Some(chrono::Utc::now().timestamp() + 1000);
        Self { aud, exp, permissions }
    }

    pub fn to_string(self, jwk: &Jwk) -> Result<String, Error> {
        let mut header: Header = Header::new(Algorithm::RS256);
        header.kid = Some(jwk.kid().to_string());
        let key: EncodingKey = EncodingKey::from_rsa_pem(jwk.private_key().as_ref())?;
        Ok(jsonwebtoken::encode(&header, &self, &key)?)
    }

    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.iter().any(|x| x == permission)
    }

    pub fn audience(&self) -> &str {
        &self.aud
    }
}
