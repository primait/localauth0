use crate::error::Error;
use crate::model::audience::Audience;
use crate::model::jwks::JwksStore;

pub struct AppData {
    audience: Audience,
    jwks_store: JwksStore,
}

impl AppData {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            audience: Audience::new(),
            jwks_store: JwksStore::new()?,
        })
    }

    pub fn audience(&self) -> &Audience {
        &self.audience
    }

    pub fn jwks_store(&self) -> &JwksStore {
        &self.jwks_store
    }
}
