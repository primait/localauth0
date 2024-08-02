use std::sync::{RwLock, RwLockWriteGuard};

use crate::error::Error;
use crate::model::{Jwk, Jwks};

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

#[cfg(test)]
mod tests {
    use crate::config::CustomField;
    use crate::error::Error;
    use crate::model::{Claims, GrantType, Jwk, Jwks};
    use crate::store::JwksStore;
    use serde_json::json;

    #[test]
    fn its_possible_to_generate_jwks_and_parse_claims_using_given_jwks_test() {
        let jwk_store: JwksStore = JwksStore::new().unwrap();
        let audience: &str = "audience";
        let permission: &str = "permission";
        let issuer: &str = "issuer";
        let subject: &str = "subject";
        let gty: GrantType = GrantType::ClientCredentials;

        let jwks: Jwks = jwk_store.get().unwrap();
        let random_jwk: Jwk = jwks.random_jwk().unwrap();

        let claims: Claims = Claims::new(
            audience.to_string(),
            vec![permission.to_string()],
            issuer.to_string(),
            subject.to_string(),
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
        let subject: &str = "subject";
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
            subject.to_string(),
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
        let subject: &str = "subject";
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
            subject.to_string(),
            gty,
            custom_claims,
        );

        let jwt: String = random_jwk.encode(&claims).unwrap();
        let result: serde_json::Value = jwks.parse(jwt.as_ref(), &[audience]).unwrap();
        assert_eq!(result.get("at_custom_claims_str").unwrap(), "my-str-2");
    }
}
