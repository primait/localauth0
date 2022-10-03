use crate::config::Config;
use crate::error::Error;
use crate::model::audience::AudiencesStore;
use crate::model::ca::CA;
use crate::model::jwks::JwksStore;

use super::authorizations::Authorizations;

pub struct AppData {
    config: Config,
    audiences_store: AudiencesStore,
    jwks_store: JwksStore,
    authorizations: Authorizations,
    ca: CA,
}

impl AppData {
    pub fn new(config: Config) -> Result<Self, Error> {
        let ca = CA::new()?;
        Ok(Self {
            audiences_store: AudiencesStore::new(config.audience()),
            jwks_store: JwksStore::new(&ca)?,
            authorizations: Authorizations::default(),
            ca,
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

    pub fn ca(&self) -> &CA {
        &self.ca
    }
}
