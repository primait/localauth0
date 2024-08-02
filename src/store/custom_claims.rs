use crate::config::{AccessTokenConfig, CustomField};
use crate::error::Error;
use std::sync::RwLock;

pub struct CustomClaims {
    cache: RwLock<Vec<CustomField>>,
}

impl Default for CustomClaims {
    fn default() -> Self {
        Self {
            cache: RwLock::new(vec![]),
        }
    }
}

impl CustomClaims {
    pub fn new(access_token: &AccessTokenConfig) -> Self {
        Self {
            cache: RwLock::new(access_token.custom_claims().clone()),
        }
    }

    pub fn add_custom_field(&self, custom_field: CustomField) -> Result<(), Error> {
        self.cache.write().unwrap_or_else(|p| p.into_inner()).push(custom_field);

        Ok(())
    }

    pub fn put_custom_fields(&self, custom_fields: Vec<CustomField>) -> Result<(), Error> {
        let mut lock = self.cache.write().unwrap_or_else(|p| p.into_inner());
        *lock = custom_fields;

        Ok(())
    }

    pub fn all(&self) -> Result<Vec<CustomField>, Error> {
        Ok(self.cache.read().unwrap_or_else(|p| p.into_inner()).clone())
    }
}
