use std::fs;

use derive_getters::Getters;
use prima_rs_logger::{error, info};
use serde::Deserialize;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Deserialize, Getters)]
pub struct Config {
    #[serde(default)]
    audience: Vec<Audience>,
    #[serde(default)]
    user: Vec<User>,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    VarError(#[from] std::env::VarError),

    #[error(transparent)]
    ReadFileError(#[from] std::io::Error),

    #[error(transparent)]
    TomlError(#[from] toml::de::Error),
}

impl Config {
    pub fn load() -> Self {
        let config: Self = match Self::load_env() {
            Ok(config) => config,
            Err(Error::VarError(_)) => {
                info!("LOCALAUTH0_CONFIG_PATH env var not set. Configuration not loaded!");
                Self {
                    audience: vec![],
                    user: vec![],
                }
            }
            Err(Error::ReadFileError(error)) => {
                error!("Failed to read file: {}", error);
                Self {
                    audience: vec![],
                    user: vec![],
                }
            }
            Err(Error::TomlError(error)) => {
                error!("Provided file not parsable: {}", error);
                Self {
                    audience: vec![],
                    user: vec![],
                }
            }
        };

        info!("Configuration is '{:?}'", &config);
        config
    }

    fn load_env() -> Result<Self> {
        let config_path: String = std::env::var("LOCALAUTH0_CONFIG_PATH")?;
        let config_string: String = fs::read_to_string(config_path)?;
        Ok(toml::from_str(config_string.as_str())?)
    }
}

#[derive(Debug, Deserialize, Getters)]
pub struct Audience {
    name: String,
    permissions: Vec<String>,
}

#[derive(Debug, Deserialize, Getters)]
pub struct User {
    name: String,
    permissions: Vec<String>,
}
