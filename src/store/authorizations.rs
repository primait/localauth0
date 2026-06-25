use std::collections::HashMap;
use std::sync::RwLock;

use crate::error::Error;

/// Data bound to an authorization code. Looked up when a client exchanges
/// the code at `POST /oauth/token` (grant_type=authorization_code) to mint
/// the id_token + access_token.
///
/// `user_email` is `None` for legacy `/oauth/login` codes (the no-credentials
/// helper used by the Yew SPA's bypass button); the token mint falls back to
/// the singleton `user_info` in that case. `Some(email)` for codes minted
/// via the full Auth0 flow at `/authorize/resume`.
#[derive(Debug, Clone, Default)]
pub struct AuthorizationData {
    pub audience: String,
    pub user_email: Option<String>,
    pub nonce: Option<String>,
}

pub struct Authorizations {
    cache: RwLock<HashMap<String, AuthorizationData>>,
}

impl Default for Authorizations {
    fn default() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
        }
    }
}

impl Authorizations {
    /// Legacy helper — returns just the audience. Kept for `/oauth/login` callers.
    pub fn get_audience_for_authorization(&self, code: &str) -> Result<String, Error> {
        Ok(self
            .cache
            .read()
            .unwrap_or_else(|p| p.into_inner())
            .get(code)
            .map(|d| d.audience.clone())
            .unwrap_or_default())
    }

    pub fn get_data_for_authorization(&self, code: &str) -> Result<Option<AuthorizationData>, Error> {
        Ok(self.cache.read().unwrap_or_else(|p| p.into_inner()).get(code).cloned())
    }

    /// Legacy helper — used by `/oauth/login` (no user, no nonce).
    pub fn put_authorization(&self, code: &str, audience: String) -> Result<(), Error> {
        let data = AuthorizationData {
            audience,
            user_email: None,
            nonce: None,
        };
        self.cache
            .write()
            .unwrap_or_else(|p| p.into_inner())
            .insert(code.to_string(), data);

        Ok(())
    }

    pub fn put_authorization_data(&self, code: &str, data: AuthorizationData) -> Result<(), Error> {
        self.cache
            .write()
            .unwrap_or_else(|p| p.into_inner())
            .insert(code.to_string(), data);
        Ok(())
    }

    pub fn all(&self) -> Result<HashMap<String, AuthorizationData>, Error> {
        Ok(self.cache.read().unwrap_or_else(|p| p.into_inner()).clone())
    }
}
