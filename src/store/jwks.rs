use std::sync::{RwLock, RwLockWriteGuard};

use crate::error::Error;
use crate::model::{Jwk, Jwks};

pub struct JwksStore {
    /// When `Some((pem, kid))`, the JWKS holds a pinned key in slot 0 that
    /// survives `/rotate` and `/revoke`. Test fixtures (and any consumer with
    /// a long-lived JWKS cache like LC MCP) can rely on the kid staying stable
    /// across container restarts.
    pinned: Option<(Vec<u8>, String)>,
    cache: RwLock<Jwks>,
}

impl JwksStore {
    pub fn new(pinned: Option<(Vec<u8>, String)>) -> Result<Self, Error> {
        let initial = match &pinned {
            Some((pem, kid)) => Jwks::new_with_pinned(pem, kid.clone())?,
            None => Jwks::new()?,
        };
        Ok(Self {
            pinned,
            cache: RwLock::new(initial),
        })
    }

    pub fn get(&self) -> Result<Jwks, Error> {
        Ok(self.cache.read().unwrap_or_else(|p| p.into_inner()).clone())
    }

    pub fn random_jwk(&self) -> Result<Jwk, Error> {
        self.cache.read().unwrap_or_else(|p| p.into_inner()).random_jwk()
    }

    /// Rotate the random-slot keys while preserving the pinned key (if any) in
    /// slot 0. Without a pinned key this behaves identically to the upstream
    /// `Jwks::rotate_keys`: prepend a fresh key and drop the oldest.
    pub fn rotate_keys(&self) -> Result<(), Error> {
        let mut jwks: RwLockWriteGuard<Jwks> = self.cache.write().unwrap_or_else(|p| p.into_inner());
        if self.pinned.is_some() {
            // Keep slot 0 (pinned). Add a fresh random in slot 1, push the rest down,
            // drop the oldest. Total stays at 3.
            let mut keys = jwks.keys.clone();
            keys.insert(1, Jwk::new()?);
            keys.pop();
            *jwks = Jwks { keys };
        } else {
            *jwks = jwks.rotate_keys()?;
        }
        Ok(())
    }

    /// Revoke and regenerate. Pinned key is re-loaded from the original PEM.
    pub fn revoke_keys(&self) -> Result<(), Error> {
        let mut jwks: RwLockWriteGuard<Jwks> = self.cache.write().unwrap_or_else(|p| p.into_inner());
        *jwks = match &self.pinned {
            Some((pem, kid)) => Jwks::new_with_pinned(pem, kid.clone())?,
            None => jwks.revoke_keys()?,
        };
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

    fn random_test_pem() -> Vec<u8> {
        // Generate a fresh RSA-2048 PEM in-memory for tests so we never touch disk.
        let rsa = openssl::rsa::Rsa::generate(2048).unwrap();
        let pkey = openssl::pkey::PKey::from_rsa(rsa).unwrap();
        pkey.private_key_to_pem_pkcs8().unwrap()
    }

    #[test]
    fn pinned_key_is_present_in_jwks_at_startup() {
        let pem = random_test_pem();
        let kid = "ct-pinned-kid".to_string();
        let store = JwksStore::new(Some((pem, kid.clone()))).unwrap();

        let jwks = store.get().unwrap();
        assert_eq!(jwks.keys.len(), 3);
        assert!(
            jwks.keys.iter().any(|k| k.kid() == kid),
            "Pinned kid {:?} not in JWKS",
            kid
        );
    }

    #[test]
    fn pinned_key_survives_rotate_and_revoke() {
        let pem = random_test_pem();
        let kid = "ct-stable-2026".to_string();
        let store = JwksStore::new(Some((pem, kid.clone()))).unwrap();

        // Three rotates would normally cycle out every initial kid.
        store.rotate_keys().unwrap();
        store.rotate_keys().unwrap();
        store.rotate_keys().unwrap();
        let jwks_after_rotate = store.get().unwrap();
        assert!(
            jwks_after_rotate.keys.iter().any(|k| k.kid() == kid),
            "Pinned kid should survive 3 rotates; kids were {:?}",
            jwks_after_rotate.keys.iter().map(|k| k.kid()).collect::<Vec<_>>()
        );
        assert_eq!(jwks_after_rotate.keys.len(), 3);

        // Revoke replaces all keys; pinned should still be present.
        store.revoke_keys().unwrap();
        let jwks_after_revoke = store.get().unwrap();
        assert!(
            jwks_after_revoke.keys.iter().any(|k| k.kid() == kid),
            "Pinned kid should survive revoke; kids were {:?}",
            jwks_after_revoke.keys.iter().map(|k| k.kid()).collect::<Vec<_>>()
        );
        assert_eq!(jwks_after_revoke.keys.len(), 3);
    }

    #[test]
    fn pinned_key_signs_and_verifies_jwts() {
        let pem = random_test_pem();
        let kid = "ct-verify-kid".to_string();
        let store = JwksStore::new(Some((pem, kid.clone()))).unwrap();

        let jwks = store.get().unwrap();
        let pinned_jwk = jwks.find(kid.clone()).expect("pinned kid must be in JWKS");

        let claims = Claims::new(
            "audience".to_string(),
            vec!["permission".to_string()],
            "issuer".to_string(),
            "subject".to_string(),
            GrantType::ClientCredentials,
            vec![],
        );
        let jwt = pinned_jwk.encode(&claims).unwrap();
        let parsed: Claims = jwks.parse(&jwt, &["audience"]).unwrap();
        assert_eq!(parsed.audience(), "audience");
    }

    #[test]
    fn jwks_store_without_pinned_key_keeps_legacy_rotate_behaviour() {
        let store = JwksStore::new(None).unwrap();
        let kids_before = store
            .get()
            .unwrap()
            .keys
            .iter()
            .map(|k| k.kid().to_string())
            .collect::<Vec<_>>();

        // 3 rotates cycle every original kid out.
        store.rotate_keys().unwrap();
        store.rotate_keys().unwrap();
        store.rotate_keys().unwrap();
        let kids_after = store
            .get()
            .unwrap()
            .keys
            .iter()
            .map(|k| k.kid().to_string())
            .collect::<Vec<_>>();

        for kid in &kids_before {
            assert!(
                !kids_after.contains(kid),
                "Without pinning, all original kids should be gone after 3 rotates; {} survived",
                kid
            );
        }
    }

    #[test]
    fn its_possible_to_generate_jwks_and_parse_claims_using_given_jwks_test() {
        let jwk_store: JwksStore = JwksStore::new(None).unwrap();
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
        let jwk_store: JwksStore = JwksStore::new(None).unwrap();
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
        let jwk_store: JwksStore = JwksStore::new(None).unwrap();
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
