use jsonwebtoken::{Algorithm, DecodingKey, Header, Validation};
use std::str::FromStr;
use std::sync::RwLock;

use openssl::pkey::Private;
use openssl::rsa::Rsa;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::Error;
use crate::model::Claims;

pub struct JwksStore {
    cache: RwLock<Jwks>,
}

impl JwksStore {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            cache: RwLock::new(Jwks::new()?),
        })
    }

    pub fn get(&self) -> Result<Jwks, Error> {
        Ok(self.cache.read().unwrap_or_else(|p| p.into_inner()).clone())
    }

    pub fn get_random(&self) -> Result<Jwk, Error> {
        self.cache.read().unwrap_or_else(|p| p.into_inner()).get_random()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Jwks {
    pub keys: Vec<Jwk>,
}

impl Jwks {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            keys: (1..=3).into_iter().map(|_| Jwk::new(None)).collect::<Result<Vec<Jwk>, Error>>()?,
        })
    }

    pub fn find(&self, kid: String) -> Option<Jwk> {
        match self.keys.iter().find(|jwk| jwk.kid == kid) {
            None => None,
            Some(jwk) => Some(jwk.clone()),
        }
    }

    pub fn get_random(&self) -> Result<Jwk, Error> {
        self.keys
            .choose(&mut rand::thread_rng())
            .ok_or_else(|| Error::EmptyJwks)
            .map(|jwks| jwks.clone())
    }

    fn validate(&self, token: &str, audience: &[impl ToString]) -> Result<Claims, Error> {
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
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Jwk {
    kty: String,
    n: String,
    e: String,
    alg: String,
    kid: String,
    r#use: String,
    #[serde(skip_serializing)]
    private_key: String,
}

impl Jwk {
    pub fn new(rsa: Option<Rsa<Private>>) -> Result<Jwk, Error> {
        let rsa: Rsa<Private> = match rsa {
            Some(rsa) => rsa,
            None => Rsa::generate(2048)?,
        };

        Ok(Jwk {
            kty: "RSA".to_string(),
            n: base64_url::encode(&rsa.n().to_string()),
            e: "AQAB".to_string(),
            alg: "RS256".to_string(),
            kid: Uuid::new_v4().to_string(),
            r#use: "sig".to_string(),
            private_key: String::from_utf8(rsa.private_key_to_pem()?)?,
        })
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

    pub fn private_key(&self) -> &str {
        &self.private_key
    }

    pub fn rsa(&self) -> Result<Rsa<Private>, Error> {
        Ok(Rsa::private_key_from_pem(&self.private_key.as_bytes())?)
    }
}

#[cfg(test)]
mod tests {
    use crate::model::jwks::JwksStore;
    use crate::model::{Claims, Jwks};

    #[test]
    fn its_possible_to_generate_public_and_private_keys_test() {
        let jwk = JwksStore::new_jwk().unwrap();
        let jwks: Jwks = Jwks {
            keys: vec![jwk.clone()],
        };

        let audience: &str = "ciao";

        let jwt = Claims::new(audience.to_string(), vec![]).to_string(&jwk).unwrap();

        let c = jwks.validate(jwt.as_ref(), audience.as_bytes());
        dbg!(c);

        assert!(false);
    }
}
