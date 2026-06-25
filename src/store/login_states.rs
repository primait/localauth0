use std::collections::HashMap;
use std::sync::RwLock;

use crate::error::Error;

/// State that ties together the multi-step Auth0 login flow:
///   `GET /authorize` → `POST /u/login/identifier` → `POST /u/login/password` → `GET /authorize/resume`
///
/// A `state_token` is generated at `/authorize`, threaded through every subsequent
/// request as a query parameter, and consumed at `/authorize/resume`.
///
/// `caller_state` is the OIDC `state` parameter the JVM client passed in to
/// `/authorize`; it must be echoed back unchanged at the final redirect to
/// `redirect_uri`. Distinct from the internal `state_token`.
#[derive(Debug, Clone)]
pub struct LoginState {
    pub client_id: String,
    pub audience: String,
    pub redirect_uri: String,
    pub scope: Option<String>,
    pub caller_state: Option<String>,
    pub nonce: Option<String>,
    pub username: Option<String>,
}

pub struct LoginStates {
    cache: RwLock<HashMap<String, LoginState>>,
}

impl Default for LoginStates {
    fn default() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
        }
    }
}

impl LoginStates {
    pub fn put(&self, state_token: String, state: LoginState) -> Result<(), Error> {
        self.cache
            .write()
            .unwrap_or_else(|p| p.into_inner())
            .insert(state_token, state);
        Ok(())
    }

    pub fn get(&self, state_token: &str) -> Result<Option<LoginState>, Error> {
        Ok(self
            .cache
            .read()
            .unwrap_or_else(|p| p.into_inner())
            .get(state_token)
            .cloned())
    }

    pub fn set_username(&self, state_token: &str, username: String) -> Result<(), Error> {
        if let Some(state) = self
            .cache
            .write()
            .unwrap_or_else(|p| p.into_inner())
            .get_mut(state_token)
        {
            state.username = Some(username);
        }
        Ok(())
    }

    /// Consume the login state. Subsequent gets return None.
    pub fn take(&self, state_token: &str) -> Result<Option<LoginState>, Error> {
        Ok(self
            .cache
            .write()
            .unwrap_or_else(|p| p.into_inner())
            .remove(state_token))
    }
}
