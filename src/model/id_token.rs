use chrono::Utc;
use serde::Serialize;

use crate::config::Config;

use super::UserInfo;

#[derive(Debug, Serialize)]
pub struct IdTokenClaims<'s> {
    iss: String,
    aud: String,
    sid: String,
    #[serde(flatten)]
    user_info: UserInfo<'s>,
    iat: Option<i64>,
    exp: Option<i64>,
    nonce: Option<String>,
}

impl<'s> IdTokenClaims<'s> {
    pub fn new(config: &'s Config, audience: String, nonce: Option<String>) -> Self {
        Self {
            iss: config.issuer().clone(),
            aud: audience,
            sid: "session_id".to_string(),
            user_info: UserInfo::new(config),
            iat: Some(Utc::now().timestamp()),
            exp: Some(Utc::now().timestamp() + 60000),
            nonce,
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{SecondsFormat, Utc};
    use serde_json::{json, Value};

    use crate::config::Config;
    use crate::model::IdTokenClaims;

    #[test]
    fn id_token_serialization() {
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

        [[audience]]
        name = "audience1"
        permissions = ["audience1:permission1", "audience1:permission2"]

        [[audience]]
        name = "audience2"
        permissions = ["audience2:permission2"]
        "#;

        let config: Config = toml::from_str(config_str).unwrap();

        let audience = "audience".to_string();
        let nonce = Some("nonce".to_string());

        let user_info: IdTokenClaims = IdTokenClaims::new(&config, audience.clone(), nonce.clone());
        let now = Utc::now();
        let value: Value = serde_json::to_value(user_info).unwrap();

        let asserted: Value = json!({
            "iss": config.issuer(),
            "aud": audience,
            "sid": "session_id",
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
            "iat": now.timestamp(),
            "exp": now.timestamp() + 60000,
            "nonce": nonce,
        });

        assert_eq!(value, asserted);
    }
}
