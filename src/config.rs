use std::fs;

use derive_getters::Getters;
use serde::Deserialize;

const DEFAULT_LOCALAUTH0_CONFIG_PATH: &str = "./localauth0.toml";

#[derive(Debug, Deserialize, Getters)]
pub struct Config {
    audience: Vec<Audience>,
}

// TODO: Return a proper Error
impl Config {
    pub fn load() -> Result<Self, String> {
        let config_path = std::env::var("LOCALAUTH0_CONFIG_PATH").map_err(|err| err.to_string())?;
        let config_string = fs::read_to_string(config_path)
            .or_else(|_| fs::read_to_string(DEFAULT_LOCALAUTH0_CONFIG_PATH))
            .map_err(|err| err.to_string())?;
        Ok(toml::from_str(config_string.as_str()).map_err(|err| err.to_string())?)
    }
}

#[derive(Debug, Deserialize, Getters)]
pub struct Audience {
    name: String,
    permissions: Vec<String>,
}
