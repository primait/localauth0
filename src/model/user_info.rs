use chrono::Utc;
use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};

use crate::config::{Config, CustomField, CustomFieldValue};

#[derive(Debug)]
pub struct UserInfo<'s> {
    custom_fields: &'s [CustomField],
    sub: String,
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

impl<'s> UserInfo<'s> {
    pub fn new(config: &'s Config, audience: String) -> Self {
        Self {
            custom_fields: config.user_info().custom_fields(),
            sub: config.user_info().subject().to_string(),
            aud: audience,
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

impl<'s> Serialize for UserInfo<'s> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Should be serialized as map being that keys should be statically defined to serialize as
        // struct
        let mut map = serializer.serialize_map(None)?;

        map.serialize_entry("sub", &self.sub)?;
        map.serialize_entry("aud", &self.aud)?;
        map.serialize_entry("iat", &self.iat)?;
        map.serialize_entry("exp", &self.exp)?;
        map.serialize_entry("iss", &self.iss)?;
        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("given_name", &self.given_name)?;
        map.serialize_entry("family_name", &self.family_name)?;
        map.serialize_entry("gender", &self.gender)?;
        map.serialize_entry("birthdate", &self.birthdate)?;
        map.serialize_entry("email", &self.email)?;
        map.serialize_entry("picture", &self.picture)?;

        for custom_field in self.custom_fields {
            match custom_field.value() {
                CustomFieldValue::String(string) => map.serialize_entry(custom_field.name(), &string),
                CustomFieldValue::Vec(vec) => map.serialize_entry(custom_field.name(), &vec),
            }?;
        }

        map.end()
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
        subject = "subject"
        name = "name"
        given_name = "given_name"
        family_name = "family_name"
        gender = "gender"
        birthdate = "birthdate"
        email = "email"
        picture = "picture"
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
        "#;

        let config: Config = toml::from_str(config_str).unwrap();

        let user_info: UserInfo = UserInfo::new(&config, "audience".to_string());
        let value: Value = serde_json::to_value(user_info).unwrap();

        let asserted: Value = json!({
            "sub": config.user_info().subject(),
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
            "custom_field_str": "str",
            "custom_field_vec": ["vec"]
        });

        assert_eq!(value, asserted);
    }
}
