use std::fs;

use derive_getters::Getters;
use prima_rs_logger::{error, info};
use serde::Deserialize;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

const DEFAULT_ISSUER: &str = "https://prima.localauth0.com/";

#[derive(Debug, Deserialize, Getters, Clone)]
pub struct Config {
    #[serde(default = "default_issuer")]
    issuer: String,
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
        match Self::load_env() {
            Ok(config) => {
                info!("Configuration is '{:?}'", &config);
                config
            }
            Err(error) => {
                log_error(error);
                Self {
                    issuer: default_issuer(),
                    audience: vec![],
                    user: vec![],
                }
            }
        }
    }

    fn load_env() -> Result<Self> {
        let config_path: String = std::env::var("LOCALAUTH0_CONFIG_PATH")?;
        let config_string: String = fs::read_to_string(config_path)?;
        Ok(toml::from_str(config_string.as_str())?)
    }
}

fn default_issuer() -> String {
    DEFAULT_ISSUER.to_string()
}

#[derive(Debug, Deserialize, Getters, Clone)]
pub struct Audience {
    name: String,
    permissions: Vec<String>,
}

#[derive(Debug, Deserialize, Getters, Clone)]
pub struct User {
    name: String,
    permissions: Vec<String>,
}

fn log_error(error: Error) {
    match error {
        Error::VarError(_) => {
            info!("LOCALAUTH0_CONFIG_PATH env var not set. Configuration not loaded!");
        }
        Error::TomlError(error) => {
            error!("Provided file not parsable: {}", error);
        }
        Error::ReadFileError(error) => {
            error!("Failed to read file: {}", error);
        }
    }
}
