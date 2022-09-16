use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::config::Config;

#[derive(Serialize, Deserialize, Debug)]
pub struct UserInfo {
    aud: String,
    iat: Option<i64>,
    exp: Option<i64>,
    iss: String,
    name: String,
    given_name: String,
    family_name: String,
    gender: String,
    birthdate: String,
    email: String,
    picture: String,
}

impl UserInfo {
    pub fn from_config(config: &Config, aud: String) -> Self {
        Self {
            aud,
            iat: Some(Utc::now().timestamp()),
            exp: Some(Utc::now().timestamp() + 60000),
            iss: config.issuer().to_string(),
            name: config.user_info().name().to_string(),
            given_name: config.user_info().given_name().to_string(),
            family_name: config.user_info().family_name().to_string(),
            gender: config.user_info().gender().to_string(),
            birthdate: config.user_info().birthdate().to_string(),
            email: config.user_info().email().to_string(),
            picture: config.user_info().picture().to_string(),
        }
    }
}
