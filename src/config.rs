use std::fs;

use derive_getters::Getters;
use serde::Deserialize;
use thiserror::Error;

const DEFAULT_LOCALAUTH0_CONFIG_PATH: &str = "./localauth0.toml";

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Deserialize, Getters)]
pub struct Config {
    audience: Vec<Audience>,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to read config file: '{0}'")]
    FileReadError(String),

    #[error("Failed to parse config file: '{0}'")]
    ConfigParseError(String),
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = std::env::var("LOCALAUTH0_CONFIG_PATH").unwrap_or(DEFAULT_LOCALAUTH0_CONFIG_PATH.to_string());
        let config_string = fs::read_to_string(config_path).map_err(|err| Error::FileReadError(err.to_string()))?;
        Ok(toml::from_str(config_string.as_str()).map_err(|err| Error::ConfigParseError(err.to_string()))?)
    }
}

#[derive(Debug, Deserialize, Getters)]
pub struct Audience {
    name: String,
    permissions: Vec<String>,
}
