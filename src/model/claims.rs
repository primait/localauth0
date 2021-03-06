use std::str::FromStr;

use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::model::{Jwk, Jwks};

#[derive(Serialize, Deserialize, Debug)]
pub struct Claims {
    aud: String,
    iat: Option<i64>,
    exp: Option<i64>,
    scope: String,
    gty: String,
    #[serde(default)]
    permissions: Vec<String>,
}

impl Claims {
    pub fn new(aud: String, permissions: Vec<String>) -> Self {
        Self {
            aud,
            iat: Some(chrono::Utc::now().timestamp()),
            exp: Some(chrono::Utc::now().timestamp() + 60000),
            scope: permissions.join(" "),
            gty: "client-credentials".to_string(),
            permissions,
        }
    }

    pub fn parse(token: &str, audience: &[impl ToString], jwks: &Jwks) -> Result<Self, Error> {
        let header: Header = jsonwebtoken::decode_header(token)?;

        if let Some(jwk) = header.kid.and_then(|kid| jwks.find(kid)) {
            let mut validation: Validation = Validation::new(Algorithm::from_str(jwk.alg())?);

            if !audience.is_empty() {
                validation.set_audience(audience);
            }

            let decoding_key: DecodingKey = DecodingKey::from_rsa_components(jwk.modulus(), jwk.exponent())?;
            Ok(jsonwebtoken::decode(token, &decoding_key, &validation)?.claims)
        } else {
            Err(Error::JwtMissingKid)
        }
    }

    pub fn to_string(self, jwk: &Jwk) -> Result<String, Error> {
        let mut header: Header = Header::new(Algorithm::RS256);
        header.kid = Some(jwk.kid().to_string());
        let key: EncodingKey = EncodingKey::from_rsa_pem(jwk.private_key_pem())?;
        Ok(jsonwebtoken::encode(&header, &self, &key)?)
    }

    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.iter().any(|x| x == permission)
    }

    pub fn audience(&self) -> &str {
        &self.aud
    }
}
