use std::collections::HashMap;
use std::sync::RwLock;

use crate::config::UserConfig;
use crate::error::Error;

/// In-memory user table keyed by email. Seeded from `[[user]]` blocks in the
/// TOML config at startup; mutable at runtime via the admin `/admin/users`
/// endpoints (added separately if needed).
///
/// Passwords are stored verbatim because this is a test fixture, not a real
/// IdP. Comparing equality vs. real Auth0's bcrypt is intentionally absent.
pub struct Users {
    cache: RwLock<HashMap<String, UserConfig>>,
}

impl Users {
    pub fn new(users: &[UserConfig]) -> Self {
        let map = users.iter().map(|u| (u.email().to_string(), u.clone())).collect();
        Self {
            cache: RwLock::new(map),
        }
    }

    /// Look up a user by email. Returns `None` if no user is registered.
    pub fn get(&self, email: &str) -> Result<Option<UserConfig>, Error> {
        Ok(self.cache.read().unwrap_or_else(|p| p.into_inner()).get(email).cloned())
    }

    /// Verify a user's password. Returns the user on success, `None` otherwise.
    /// Constant-time comparison is deliberately omitted — this is for test fixtures.
    pub fn authenticate(&self, email: &str, password: &str) -> Result<Option<UserConfig>, Error> {
        Ok(self
            .cache
            .read()
            .unwrap_or_else(|p| p.into_inner())
            .get(email)
            .filter(|u| u.password() == password)
            .cloned())
    }

    /// Insert or replace a user.
    pub fn put(&self, user: UserConfig) -> Result<(), Error> {
        self.cache
            .write()
            .unwrap_or_else(|p| p.into_inner())
            .insert(user.email().to_string(), user);
        Ok(())
    }
}
