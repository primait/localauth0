use std::collections::HashMap;
use std::sync::RwLock;

use crate::error::Error;

pub struct Audience {
    cache: RwLock<HashMap<String, Vec<String>>>,
}

impl Audience {
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
        }
    }

    pub fn get_permissions(&self, audience: &str) -> Result<Vec<String>, Error> {
        Ok(self
            .cache
            .read()
            .unwrap_or_else(|p| p.into_inner())
            .get(audience)
            .map(|v| v.clone())
            .unwrap_or_default())
    }

    pub fn set_permissions(&self, audience: &str, permissions: Vec<String>) -> Result<(), Error> {
        self.cache
            .write()
            .unwrap_or_else(|p| p.into_inner())
            .insert(audience.to_string(), permissions);

        Ok(())
    }

    pub fn all(&self) -> Result<HashMap<String, Vec<String>>, Error> {
        Ok(self.cache.read().unwrap_or_else(|p| p.into_inner()).clone())
    }
}
