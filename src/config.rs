use std::fs;

use chrono::{DateTime, Utc};
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

    #[serde(default)]
    access_token: AccessToken,

    #[serde(default)]
    http: Http,

    #[serde(default)]
    https: Https,
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
                    access_token: Default::default(),
                    http: Default::default(),
                    https: Default::default(),
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
    #[serde(default = "defaults::user_info_subject")]
    subject: String,
    #[serde(default = "defaults::user_info_name")]
    name: String,
    #[serde(default = "defaults::user_info_given_name")]
    given_name: String,
    #[serde(default = "defaults::user_info_family_name")]
    family_name: String,
    #[serde(default = "defaults::user_info_nickname")]
    nickname: String,
    #[serde(default = "defaults::user_info_locale")]
    locale: String,
    #[serde(default = "defaults::user_info_gender")]
    gender: String,
    #[serde(default = "defaults::user_info_birthdate")]
    birthdate: String,
    #[serde(default = "defaults::user_info_email")]
    email: String,
    #[serde(default = "defaults::user_info_email_verified")]
    email_verified: bool,
    #[serde(default = "defaults::user_info_picture")]
    picture: String,
    #[serde(default = "defaults::user_info_updated_at")]
    updated_at: DateTime<Utc>,
    #[serde(default)]
    custom_fields: Vec<CustomField>,
}

impl Default for UserInfo {
    fn default() -> Self {
        Self {
            subject: defaults::user_info_subject(),
            name: defaults::user_info_name(),
            given_name: defaults::user_info_given_name(),
            family_name: defaults::user_info_family_name(),
            nickname: defaults::user_info_nickname(),
            locale: defaults::user_info_locale(),
            gender: defaults::user_info_gender(),
            birthdate: defaults::user_info_birthdate(),
            email: defaults::user_info_email(),
            email_verified: defaults::user_info_email_verified(),
            picture: defaults::user_info_picture(),
            updated_at: defaults::user_info_updated_at(),
            custom_fields: vec![],
        }
    }
}

#[derive(Debug, Deserialize, Getters, Default)]
pub struct AccessToken {
    #[serde(default)]
    custom_claims: Vec<CustomField>,
}

#[derive(Debug, Deserialize, Getters, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct CustomField {
    name: String,
    value: CustomFieldValue,
}

impl CustomField {
    pub fn new(name: String, value: CustomFieldValue) -> Self {
        Self { name, value }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub enum CustomFieldValue {
    String(String),
    Vec(Vec<String>),
}

#[derive(Debug, Deserialize, Getters)]
pub struct Http {
    port: u16,
}

impl Default for Http {
    fn default() -> Self {
        Http {
            port: defaults::http_port(),
        }
    }
}

#[derive(Debug, Deserialize, Getters)]
pub struct Https {
    port: u16,
}

impl Default for Https {
    fn default() -> Self {
        Https {
            port: defaults::https_port(),
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
    use chrono::DateTime;

    use crate::config::{Audience, Config, CustomField, CustomFieldValue};
    #[test]
    fn local_localauth0_config_file_is_loadable() {
        let config_string: String = std::fs::read_to_string("./localauth0.toml").unwrap();
        let loaded_config_result: Result<Config, toml::de::Error> = toml::from_str(config_string.as_str());
        assert!(loaded_config_result.is_ok())
    }

    #[test]
    fn localauth0_config_is_loadable() {
        let config_str: &str = r#"
        issuer = "issuer"

        [user_info]
        subject = "subject"
        name = "name"
        given_name = "given_name"
        family_name = "family_name"
        nickname = "nickname"
        locale = "locale"
        gender = "gender"
        birthdate = "birthdate"
        email = "email"
        email_verified = true
        picture = "picture"
        updated_at = "2022-11-11T11:00:00Z"
        custom_fields = [
            { name = "custom_field_str", value = { String = "str" } },
            { name = "custom_field_vec", value = { Vec = ["vec"] } }
        ]

        [[audience]]
        name = "audience1"
        permissions = ["audience1:permission1", "audience1:permission2"]

        [[audience]]
        name = "audience2"
        permissions = ["audience2:permission2"]

        [access_token]
        custom_claims = [
            { name = "at_custom_claim_str", value = { String = "str" } }
        ]

        [http]
        port = 8000
        "#;

        let config: Config = toml::from_str(config_str).unwrap();

        assert_eq!(config.issuer(), "issuer");

        assert_eq!(config.user_info().subject, "subject");
        assert_eq!(config.user_info().name, "name");
        assert_eq!(config.user_info().given_name, "given_name");
        assert_eq!(config.user_info().family_name, "family_name");
        assert_eq!(config.user_info().nickname, "nickname");
        assert_eq!(config.user_info().locale, "locale");
        assert_eq!(config.user_info().gender, "gender");
        assert_eq!(config.user_info().birthdate, "birthdate");
        assert_eq!(config.user_info().email, "email");
        assert!(config.user_info().email_verified);
        assert_eq!(
            config.user_info().updated_at,
            DateTime::parse_from_rfc3339("2022-11-11T11:00:00Z").unwrap()
        );
        assert_eq!(config.user_info().picture, "picture");

        assert_eq!(config.audience.len(), 2);

        let audience1: Option<&Audience> = config.audience.iter().find(|v| v.name == "audience1");
        assert!(audience1.is_some());
        assert_eq!(
            audience1.unwrap().permissions,
            ["audience1:permission1", "audience1:permission2"]
        );

        let audience2: Option<&Audience> = config.audience.iter().find(|v| v.name == "audience2");
        assert!(audience2.is_some());
        assert_eq!(audience2.unwrap().permissions, ["audience2:permission2"]);

        let custom_fields: &Vec<CustomField> = config.user_info().custom_fields();

        assert_eq!(custom_fields.len(), 2);

        let custom_field: &CustomField = custom_fields.iter().find(|v| v.name == "custom_field_vec").unwrap();
        assert_eq!(custom_field.value, CustomFieldValue::Vec(vec!["vec".to_string()]));

        let custom_field: &CustomField = custom_fields.iter().find(|v| v.name == "custom_field_str").unwrap();
        assert_eq!(custom_field.value, CustomFieldValue::String("str".to_string()));

        let access_token = config.access_token();
        let at_custom_claims = access_token.custom_claims();

        let at_custom_claim: &CustomField = at_custom_claims
            .iter()
            .find(|v| v.name == "at_custom_claim_str")
            .unwrap();
        assert_eq!(at_custom_claim.value, CustomFieldValue::String("str".to_string()));

        assert_eq!(&8000, config.http().port());
        assert_eq!(&3001, config.https().port());
    }
}
