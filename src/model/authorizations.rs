use std::collections::HashMap;
use std::sync::RwLock;

use crate::error::Error;

pub struct Authorizations {
    cache: RwLock<HashMap<String, String>>,
}

impl Default for Authorizations {
    fn default() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
        }
    }
}

impl Authorizations {
    pub fn get_audience_for_authorization(&self, code: &str) -> Result<String, Error> {
        Ok(self
            .cache
            .read()
            .unwrap_or_else(|p| p.into_inner())
            .get(code)
            .cloned()
            .unwrap_or_default())
    }

    pub fn put_authorization(&self, code: &str, audience: String) -> Result<(), Error> {
        self.cache
            .write()
            .unwrap_or_else(|p| p.into_inner())
            .insert(code.to_string(), audience);

        Ok(())
    }

    pub fn all(&self) -> Result<HashMap<String, String>, Error> {
        Ok(self.cache.read().unwrap_or_else(|p| p.into_inner()).clone())
    }
}
