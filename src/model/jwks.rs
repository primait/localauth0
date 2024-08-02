use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use openssl::pkey::Private;
use openssl::rsa::Rsa;
use rand::seq::SliceRandom;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

use crate::error::Error;
use crate::model::certificates;

#[derive(Serialize, Deserialize, Clone)]
pub struct Jwks {
    pub keys: Vec<Jwk>,
}

impl Jwks {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            keys: (1..=3).map(|_| Jwk::new()).collect::<Result<Vec<Jwk>, Error>>()?,
        })
    }

    pub fn find(&self, kid: String) -> Option<Jwk> {
        self.keys.iter().find(|jwk| jwk.kid == kid).cloned()
    }

    pub fn random_jwk(&self) -> Result<Jwk, Error> {
        self.keys
            .choose(&mut rand::thread_rng())
            .ok_or(Error::EmptyJwks)
            .cloned()
    }

    pub fn parse<T: DeserializeOwned>(&self, token: &str, audience: &[impl ToString]) -> Result<T, Error> {
        let header: Header = jsonwebtoken::decode_header(token)?;

        if let Some(jwk) = header.kid.and_then(|kid| self.find(kid)) {
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

    pub fn rotate_keys(&self) -> Result<Self, Error> {
        let mut keys: Vec<Jwk> = self.keys.clone();
        keys.insert(0, Jwk::new()?);
        keys.pop();
        Ok(Self { keys })
    }

    pub fn revoke_keys(&self) -> Result<Self, Error> {
        Jwks::new()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Jwk {
    kty: String,
    n: String,
    e: String,
    alg: String,
    kid: String,
    r#use: String,
    x5c: Vec<String>,
    #[serde(skip_serializing, skip_deserializing)]
    private_key_pem: Vec<u8>,
}

impl Jwk {
    pub fn new() -> Result<Jwk, Error> {
        let key_pair = certificates::generate_private_key()?;
        let modulus = key_pair.rsa()?.n().to_vec();
        let exponent = key_pair.rsa()?.e().to_vec();

        let x509 = certificates::generate_certificate(&key_pair)?;
        let x509cert = BASE64_STANDARD.encode(x509.to_der()?);

        Ok(Self {
            kty: "RSA".to_string(),
            n: base64_url::encode(&modulus),
            e: base64_url::encode(&exponent),
            alg: "RS256".to_string(),
            kid: Uuid::new_v4().to_string(),
            r#use: "sig".to_string(),
            x5c: vec![x509cert],
            private_key_pem: key_pair.private_key_to_pem_pkcs8()?,
        })
    }

    pub fn encode<T: Serialize>(&self, t: &T) -> Result<String, Error> {
        let mut header: Header = Header::new(Algorithm::RS256);
        header.kid = Some(self.kid().to_string());
        let key: EncodingKey = EncodingKey::from_rsa_pem(self.private_key_pem())?;
        Ok(jsonwebtoken::encode(&header, &t, &key)?)
    }

    pub fn kid(&self) -> &str {
        &self.kid
    }

    pub fn alg(&self) -> &str {
        &self.alg
    }

    pub fn modulus(&self) -> &str {
        &self.n
    }

    pub fn exponent(&self) -> &str {
        &self.e
    }

    pub fn private_key_pem(&self) -> &Vec<u8> {
        &self.private_key_pem
    }

    pub fn rsa(&self) -> Result<Rsa<Private>, Error> {
        Ok(Rsa::private_key_from_pem(&self.private_key_pem)?)
    }
}
