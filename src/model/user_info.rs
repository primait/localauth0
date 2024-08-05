use chrono::{DateTime, SecondsFormat, Utc};
use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};

use crate::config::{CustomField, CustomFieldValue, UserInfoConfig};

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(serde::Deserialize, PartialEq))]
pub struct UserInfo {
    pub sub: String,
    pub name: String,
    pub given_name: String,
    pub family_name: String,
    pub nickname: String,
    pub locale: String,
    pub gender: String,
    pub birthdate: String,
    pub email: String,
    pub email_verified: bool,
    pub picture: String,
    pub updated_at: DateTime<Utc>,
    pub custom_fields: Option<Vec<CustomField>>,
}

impl From<&UserInfoConfig> for UserInfo {
    fn from(value: &UserInfoConfig) -> Self {
        Self {
            sub: value.subject().to_string(),
            name: value.name().to_string(),
            given_name: value.given_name().to_string(),
            family_name: value.family_name().to_string(),
            nickname: value.nickname().to_string(),
            locale: value.locale().to_string(),
            gender: value.gender().to_string(),
            birthdate: value.birthdate().to_string(),
            email: value.email().to_string(),
            email_verified: *value.email_verified(),
            picture: value.picture().to_string(),
            updated_at: *value.updated_at(),
            custom_fields: value.custom_fields().clone(),
        }
    }
}

impl Serialize for UserInfo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Should be serialized as map being that keys should be statically defined to serialize as
        // struct
        let mut map = serializer.serialize_map(None)?;

        map.serialize_entry("sub", &self.sub)?;
        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("given_name", &self.given_name)?;
        map.serialize_entry("family_name", &self.family_name)?;
        map.serialize_entry("nickname", &self.nickname)?;
        map.serialize_entry("locale", &self.locale)?;
        map.serialize_entry("gender", &self.gender)?;
        map.serialize_entry("birthdate", &self.birthdate)?;
        map.serialize_entry("email", &self.email)?;
        map.serialize_entry("email_verified", &self.email_verified)?;
        map.serialize_entry("picture", &self.picture)?;
        map.serialize_entry(
            "updated_at",
            &self.updated_at.to_rfc3339_opts(SecondsFormat::Millis, true),
        )?;

        for custom_field in self.custom_fields.as_deref().unwrap_or_default() {
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
    use chrono::SecondsFormat;
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
        nickname = "nickname"
        locale = "en"
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
        "#;

        let config: Config = toml::from_str(config_str).unwrap();

        let user_info: UserInfo = config.user_info().into();
        let value: Value = serde_json::to_value(user_info).unwrap();

        let asserted: Value = json!({
            "sub": config.user_info().subject(),
            "name": config.user_info().name(),
            "given_name": config.user_info().given_name(),
            "family_name": config.user_info().family_name(),
            "nickname": config.user_info().nickname(),
            "locale": config.user_info().locale(),
            "gender": config.user_info().gender(),
            "birthdate": config.user_info().birthdate(),
            "email": config.user_info().email(),
            "email_verified": config.user_info().email_verified(),
            "picture": config.user_info().picture(),
            "updated_at": config.user_info().updated_at().to_rfc3339_opts(SecondsFormat::Millis, true),
            "custom_field_str": "str",
            "custom_field_vec": ["vec"]
        });

        assert_eq!(value, asserted);
    }
}
