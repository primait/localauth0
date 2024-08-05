use crate::config::AudienceConfig;
use std::collections::HashMap;
use std::sync::RwLock;

use crate::error::Error;

pub struct Audiences {
    cache: RwLock<HashMap<String, Vec<String>>>,
}

impl Audiences {
    pub fn new(audiences: &[AudienceConfig]) -> Self {
        let mut map: HashMap<String, Vec<String>> = HashMap::new();

        for audience in audiences {
            map.insert(audience.name().to_string(), audience.permissions().clone());
        }

        Self {
            cache: RwLock::new(map),
        }
    }

    pub fn get_permissions(&self, audience: &str) -> Result<Vec<String>, Error> {
        Ok(self
            .cache
            .read()
            .unwrap_or_else(|p| p.into_inner())
            .get(audience)
            .cloned()
            .unwrap_or_default())
    }

    pub fn put_permissions(&self, audience: &str, permissions: Vec<String>) -> Result<(), Error> {
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
