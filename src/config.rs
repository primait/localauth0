use derive_getters::Getters;
use serde::Deserialize;

#[derive(Getters, Deserialize)]
pub struct Config {
    permission_settings: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self, envy::Error> {
        let config: Config = envy::from_env()?;
        Ok(config)
    }
}
