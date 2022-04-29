use std::collections::HashMap;
use std::sync::RwLock;

use crate::error::Error;

use super::PermissionsForAudienceRequest;

pub struct Audience {
    cache: RwLock<HashMap<String, Vec<String>>>,
}

impl Default for Audience {
    fn default() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
        }
    }
}

impl Audience {
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

    pub fn load_permission_settings(&self, permission_settings: &Option<Vec<PermissionsForAudienceRequest>>) -> () {
        if let None = permission_settings {
            return ();
        }

        match permission_settings {
            None => (),
            Some(requests) => {
                requests.iter().for_each(|request| {
                    self.put_permissions(request.audience.as_str(), request.permissions.clone())
                        .unwrap();
                });
            }
        }
    }

    pub fn all(&self) -> Result<HashMap<String, Vec<String>>, Error> {
        Ok(self.cache.read().unwrap_or_else(|p| p.into_inner()).clone())
    }
}
