use crate::config::Config;
use crate::error::Error;
use crate::model::audience::AudiencesStore;
use crate::model::jwks::JwksStore;

use super::authorizations::Authorizations;

pub struct AppData {
    config: Config,
    audiences_store: AudiencesStore,
    jwks_store: JwksStore,
    authorizations: Authorizations,
}

impl AppData {
    pub fn new(config: Config) -> Result<Self, Error> {
        Ok(Self {
            audiences_store: AudiencesStore::new(config.audience()),
            jwks_store: JwksStore::new()?,
            authorizations: Authorizations::default(),
            config,
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

    pub fn authorizations(&self) -> &Authorizations {
        &self.authorizations
    }
}
