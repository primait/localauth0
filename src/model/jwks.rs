use std::sync::{RwLock, RwLockWriteGuard};

use openssl::pkey::{PKey, Private};
use openssl::rsa::Rsa;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::Error;

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
                .into_iter()
                .map(|_| Jwk::new(None))
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

    pub fn rotate_keys(&self) -> Result<Self, Error> {
        let mut keys: Vec<Jwk> = self.keys.clone();
        keys.insert(0, Jwk::new(None)?);
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
    #[serde(skip_serializing)]
    private_key_pem: Vec<u8>,
}

impl Jwk {
    pub fn new(rsa: Option<Rsa<Private>>) -> Result<Jwk, Error> {
        let rsa: Rsa<Private> = match rsa {
            Some(rsa) => rsa,
            None => Rsa::generate(2048)?,
        };

        let modulus: Vec<u8> = rsa.n().to_vec();
        let exponent: Vec<u8> = rsa.e().to_vec();
        let pkey: PKey<Private> = PKey::from_rsa(rsa)?;

        Ok(Self {
            kty: "RSA".to_string(),
            n: base64_url::encode(&modulus),
            e: base64_url::encode(&exponent),
            alg: "RS256".to_string(),
            kid: Uuid::new_v4().to_string(),
            r#use: "sig".to_string(),
            private_key_pem: pkey.private_key_to_pem_pkcs8()?,
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

    pub fn private_key_pem(&self) -> &Vec<u8> {
        &self.private_key_pem
    }

    pub fn rsa(&self) -> Result<Rsa<Private>, Error> {
        Ok(Rsa::private_key_from_pem(&self.private_key_pem)?)
    }
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::model::jwks::JwksStore;
    use crate::model::{Claims, GrantType, Jwk, Jwks};

    #[test]
    fn its_possible_to_generate_jwks_and_parse_claims_using_given_jwks_test() {
        let jwk_store: JwksStore = JwksStore::new().unwrap();
        let audience: &str = "audience";
        let permission: &str = "permission";
        let issuer: &str = "issuer";
        let gty: GrantType = GrantType::ClientCredentials;

        let jwks: Jwks = jwk_store.get().unwrap();
        let random_jwk: Jwk = jwks.random_jwk().unwrap();

        let jwt: String = Claims::new(
            audience.to_string(),
            vec![permission.to_string()],
            issuer.to_string(),
            gty.clone(),
        )
        .to_string(&random_jwk)
        .unwrap();

        let result: Result<Claims, Error> = Claims::parse(jwt.as_ref(), &[audience], &jwks);

        assert!(result.is_ok());
        let claims: Claims = result.unwrap();
        assert_eq!(claims.audience(), audience);
        assert!(claims.has_permission(permission));
        assert_eq!(claims.issuer(), issuer);
        assert_eq!(claims.grant_type().to_string(), gty.to_string());
    }
}
