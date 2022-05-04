use std::fs;

use derive_getters::Getters;
use serde::Deserialize;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Deserialize, Getters)]
pub struct Config {
    audience: Vec<Audience>,
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
                println!("LOCALAUTH0_CONFIG_PATH env var not set. Configuration not loaded!");
                Self { audience: vec![] }
            }
            Err(Error::ReadFileError(error)) => {
                println!("Failed to read file: {}", error);
                Self { audience: vec![] }
            }
            Err(Error::TomlError(error)) => {
                println!("Provided file not parsable: {}", error);
                Self { audience: vec![] }
            }
        };

        println!("{:?}", &config);
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
