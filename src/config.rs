use derive_getters::Getters;
use serde::Deserialize;

use crate::model::PermissionsForAudienceRequest;

#[derive(Getters, Deserialize)]
pub struct Config {
    permission_settings: Option<Vec<PermissionsForAudienceRequest>>,
}

impl Config {
    pub fn load() -> Result<Self, envy::Error> {
        let config: Config = envy::from_env()?;
        Ok(config)
    }
}
