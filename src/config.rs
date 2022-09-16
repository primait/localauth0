use std::fs;

use derive_getters::Getters;
use prima_rs_logger::{error, info};
use serde::Deserialize;
use thiserror::Error;

use crate::model::defaults;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Deserialize, Getters)]
pub struct Config {
    #[serde(default = "defaults::issuer")]
    issuer: String,

    #[serde(default)]
    user_info: UserInfo,

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
                    issuer: defaults::issuer(),
                    user_info: Default::default(),
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

#[derive(Debug, Deserialize, Getters)]
pub struct UserInfo {
    #[serde(default = "defaults::user_info_name")]
    name: String,
    #[serde(default = "defaults::user_info_given_name")]
    given_name: String,
    #[serde(default = "defaults::user_info_family_name")]
    family_name: String,
    #[serde(default = "defaults::user_info_gender")]
    gender: String,
    #[serde(default = "defaults::user_info_birthdate")]
    birthdate: String,
    #[serde(default = "defaults::user_info_email")]
    email: String,
    #[serde(default = "defaults::user_info_picture")]
    picture: String,
}

impl Default for UserInfo {
    fn default() -> Self {
        Self {
            name: defaults::user_info_name(),
            given_name: defaults::user_info_given_name(),
            family_name: defaults::user_info_family_name(),
            gender: defaults::user_info_gender(),
            birthdate: defaults::user_info_birthdate(),
            email: defaults::user_info_email(),
            picture: defaults::user_info_picture(),
        }
    }
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

#[cfg(test)]
mod tests {
    use crate::config::{Config, Error};

    #[test]
    fn local_localauth0_config_is_loadable() {
        std::env::set_var("LOCALAUTH0_CONFIG_PATH", "./localauth0.toml");
        let config_result: Result<Config, Error> = Config::load_env();

        assert!(config_result.is_ok());
    }
}
