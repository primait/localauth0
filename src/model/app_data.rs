use crate::error::Error;
use crate::model::audience::AudiencesStore;
use crate::model::jwks::JwksStore;

pub struct AppData {
    audiences_store: AudiencesStore,
    jwks_store: JwksStore,
}

impl AppData {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            audiences_store: AudiencesStore::default(),
            jwks_store: JwksStore::new()?,
        })
    }

    pub fn audiences(&self) -> &AudiencesStore {
        &self.audiences_store
    }

    pub fn jwks(&self) -> &JwksStore {
        &self.jwks_store
    }
}
