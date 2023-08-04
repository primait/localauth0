use std::str::FromStr;
use std::sync::{RwLock, RwLockWriteGuard};

use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use openssl::pkey::Private;
use openssl::rsa::Rsa;
use rand::seq::SliceRandom;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::Error;
use crate::model::certificates;

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

    pub fn random_jwk(&self) -> Result<Jwk, Error> {
        self.cache.read().unwrap_or_else(|p| p.into_inner()).random_jwk()
    }

    pub fn rotate_keys(&self) -> Result<(), Error> {
        let mut jwks: RwLockWriteGuard<Jwks> = self.cache.write().unwrap_or_else(|p| p.into_inner());
        *jwks = jwks.rotate_keys()?;
        Ok(())
    }

    pub fn revoke_keys(&self) -> Result<(), Error> {
        let mut jwks: RwLockWriteGuard<Jwks> = self.cache.write().unwrap_or_else(|p| p.into_inner());
        *jwks = jwks.revoke_keys()?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Jwks {
    pub keys: Vec<Jwk>,
}

impl Jwks {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            keys: (1..=3)
                .map(|_| Jwk::new())
                .collect::<Result<Vec<Jwk>, Error>>()?,
        })
    }

    pub fn find(&self, kid: String) -> Option<Jwk> {
        self.keys.iter().find(|jwk| jwk.kid == kid).cloned()
    }

    pub fn random_jwk(&self) -> Result<Jwk, Error> {
        self.keys
            .choose(&mut rand::thread_rng())
            .ok_or(Error::EmptyJwks)
            .map(|jwks| jwks.clone())
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
    #[serde(skip_serializing)]
    private_key_pem: Vec<u8>,
}

impl Jwk {
    pub fn new() -> Result<Jwk, Error> {
        let key_pair = certificates::generate_private_key()?;
        let modulus = key_pair.rsa()?.n().to_vec();
        let exponent = key_pair.rsa()?.e().to_vec();

        let x509 = certificates::generate_certificate(&key_pair)?;
        let x509cert = base64::encode(x509.to_der()?);

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

#[cfg(test)]
mod tests {
    use crate::config::CustomField;
    use crate::error::Error;
    use crate::model::jwks::JwksStore;
    use crate::model::{Claims, GrantType, Jwk, Jwks};
    use serde_json::json;

    #[test]
    fn its_possible_to_generate_jwks_and_parse_claims_using_given_jwks_test() {
        let jwk_store: JwksStore = JwksStore::new().unwrap();
        let audience: &str = "audience";
        let permission: &str = "permission";
        let issuer: &str = "issuer";
        let gty: GrantType = GrantType::ClientCredentials;

        let jwks: Jwks = jwk_store.get().unwrap();
        let random_jwk: Jwk = jwks.random_jwk().unwrap();

        let claims: Claims = Claims::new(
            audience.to_string(),
            vec![permission.to_string()],
            issuer.to_string(),
            gty.clone(),
            vec![],
        );

        let jwt: String = random_jwk.encode(&claims).unwrap();
        let result: Result<Claims, Error> = jwks.parse(jwt.as_ref(), &[audience]);
        assert!(result.is_ok());

        let claims: Claims = result.unwrap();
        assert_eq!(claims.audience(), audience);
        assert!(claims.has_permission(permission));
        assert_eq!(claims.issuer(), issuer);
        assert_eq!(claims.grant_type().to_string(), gty.to_string());
    }

    #[test]
    fn use_custom_claims_test() {
        let jwk_store: JwksStore = JwksStore::new().unwrap();
        let audience: &str = "audience";
        let permission: &str = "permission";
        let issuer: &str = "issuer";
        let gty: GrantType = GrantType::ClientCredentials;

        let jwks: Jwks = jwk_store.get().unwrap();
        let random_jwk: Jwk = jwks.random_jwk().unwrap();
        let custom_claims: Vec<CustomField> = vec![
            serde_json::from_value(json!({ "name": "at_custom_claims_str", "value": { "String": "my_str" } })).unwrap(),
            serde_json::from_value(json!({"name": "at_custom_claims_vec", "value": {"Vec": ["foobar"]}})).unwrap(),
        ];

        let claims: Claims = Claims::new(
            audience.to_string(),
            vec![permission.to_string()],
            issuer.to_string(),
            gty,
            custom_claims,
        );

        let jwt: String = random_jwk.encode(&claims).unwrap();
        let content: serde_json::Value = jwks.parse(jwt.as_ref(), &[audience]).expect("unable to parse jwt");
        assert_eq!(content.get("at_custom_claims_str").unwrap(), "my_str");
        let custom_claim_vec: Vec<String> =
            serde_json::from_value(content.get("at_custom_claims_vec").unwrap().to_owned()).unwrap();
        assert_eq!(custom_claim_vec, vec!["foobar".to_string()]);
    }

    #[test]
    fn duplicated_custom_claim_keeps_the_last_one() {
        let jwk_store: JwksStore = JwksStore::new().unwrap();
        let audience: &str = "audience";
        let permission: &str = "permission";
        let issuer: &str = "issuer";
        let gty: GrantType = GrantType::ClientCredentials;

        let jwks: Jwks = jwk_store.get().unwrap();
        let random_jwk: Jwk = jwks.random_jwk().unwrap();
        let custom_claims: Vec<CustomField> = vec![
            serde_json::from_value(json!({ "name": "at_custom_claims_str", "value": { "String": "my-str-1" } }))
                .unwrap(),
            serde_json::from_value(json!({ "name": "at_custom_claims_str", "value": { "String": "my-str-2" } }))
                .unwrap(),
        ];

        let claims: Claims = Claims::new(
            audience.to_string(),
            vec![permission.to_string()],
            issuer.to_string(),
            gty,
            custom_claims,
        );

        let jwt: String = random_jwk.encode(&claims).unwrap();
        let result: serde_json::Value = jwks.parse(jwt.as_ref(), &[audience]).unwrap();
        assert_eq!(result.get("at_custom_claims_str").unwrap(), "my-str-2");
    }
}
