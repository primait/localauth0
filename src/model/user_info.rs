use chrono::Utc;
use serde::de::Error;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::config::{AdditionalFieldValue, Config};

pub struct UserInfo;

impl UserInfo {
    pub fn encode_to_value(config: &Config, aud: String) -> Result<Value, serde_json::Error> {
        let inner_user_info: InnerUserInfo = InnerUserInfo::from_config(config, aud);
        let value: Value = serde_json::to_value(&inner_user_info)?;
        match value {
            Value::Object(mut map) => {
                for additional_field in config.user_info().additional_fields() {
                    let name: String = additional_field.name().to_string();
                    match additional_field.value() {
                        AdditionalFieldValue::String(string) => {
                            map.insert(name, Value::String(string.to_string()));
                        }
                        AdditionalFieldValue::Vec(vec) => {
                            let values: Vec<Value> = vec
                                .iter()
                                .map(|string| Value::String(string.to_string()))
                                .collect();

                            map.insert(name, Value::Array(values));
                        }
                    }
                }
                Ok(Value::Object(map))
            }
            _ => Err(serde_json::Error::custom(
                "Something went wrong putting additional fields to user info",
            )),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct InnerUserInfo {
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

impl InnerUserInfo {
    fn from_config(config: &Config, aud: String) -> Self {
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

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use serde_json::{json, Value};

    use crate::config::Config;
    use crate::model::UserInfo;

    #[test]
    fn user_info_serialization() {
        let config_str: &str = r#"
        issuer = "issuer"

        [user_info]
        name = "name"
        given_name = "given_name"
        family_name = "family_name"
        gender = "gender"
        birthdate = "birthdate"
        email = "email"
        picture = "picture"
        additional_fields = [
            { name = "additional_str", value = { String = "str" } },
            { name = "additional_vec", value = { Vec = ["vec"] } }
        ]

        [[audience]]
        name = "audience1"
        permissions = ["audience1:permission1", "audience1:permission2"]

        [[audience]]
        name = "audience2"
        permissions = ["audience2:permission2"]
        "#;

        let config: Config = toml::from_str(config_str).unwrap();

        let value: Value = UserInfo::encode_to_value(&config, "audience".to_string()).unwrap();

        let asserted: Value = json!({
            "aud": "audience",
            "iat": Utc::now().timestamp(),
            "exp": Utc::now().timestamp() + 60000,
            "iss": config.issuer(),
            "name": config.user_info().name(),
            "given_name": config.user_info().given_name(),
            "family_name": config.user_info().family_name(),
            "gender": config.user_info().gender(),
            "birthdate": config.user_info().birthdate(),
            "email": config.user_info().email(),
            "picture": config.user_info().picture(),
            "additional_str": "str",
            "additional_vec": ["vec"]
        });

        assert_eq!(value, asserted);
    }
}
