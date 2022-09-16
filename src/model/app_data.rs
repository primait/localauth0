use crate::config::Config;
use crate::error::Error;
use crate::model::audience::AudiencesStore;
use crate::model::jwks::JwksStore;

pub struct AppData {
    config: Config,
    audiences_store: AudiencesStore,
    jwks_store: JwksStore,
}

impl AppData {
    pub fn new(config: Config) -> Result<Self, Error> {
        Ok(Self {
            config,
            audiences_store: AudiencesStore::default(),
            jwks_store: JwksStore::new()?,
        })
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn audiences(&self) -> &AudiencesStore {
        &self.audiences_store
    }

    pub fn jwks(&self) -> &JwksStore {
        &self.jwks_store
    }
}
