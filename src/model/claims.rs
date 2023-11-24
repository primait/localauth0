use std::fmt::{Display, Formatter};

use serde::{ser::SerializeMap, Deserialize, Serialize, Serializer};
use uuid::Uuid;

use crate::config::{CustomField, CustomFieldValue};

#[derive(Debug, Deserialize)]
pub struct Claims {
    iss: String,
    sub: String,
    aud: String,
    exp: Option<i64>,
    nbf: Option<i64>,
    iat: Option<i64>,
    jti: String,
    scope: String,
    gty: GrantType,
    permissions: Vec<String>,
    // skip deserializing since deserialization from a jwt wouldn't match this struct
    // a custom deserializer would be needed
    #[serde(skip_deserializing)]
    custom_claims: Vec<CustomField>,
}

impl Claims {
    pub fn new(
        aud: String,
        permissions: Vec<String>,
        iss: String,
        sub: String,
        gty: GrantType,
        custom_claims: Vec<CustomField>,
    ) -> Self {
        Self {
            iss,
            sub,
            aud,
            exp: Some(chrono::Utc::now().timestamp() + 60000),
            nbf: Some(chrono::Utc::now().timestamp()),
            iat: Some(chrono::Utc::now().timestamp()),
            jti: Uuid::new_v4().to_string(),
            scope: permissions.join(" "),
            gty,
            permissions,
            custom_claims,
        }
    }

    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.iter().any(|x| x == permission)
    }

    pub fn audience(&self) -> &str {
        &self.aud
    }

    pub fn issuer(&self) -> &str {
        &self.iss
    }

    pub fn subject(&self) -> &str {
        &self.sub
    }

    pub fn grant_type(&self) -> &GrantType {
        &self.gty
    }

    #[cfg(test)]
    pub fn custom_claims(&self) -> &Vec<CustomField> {
        &self.custom_claims
    }
}

impl Serialize for Claims {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;

        map.serialize_entry("iss", &self.iss)?;
        map.serialize_entry("sub", &self.sub)?;
        map.serialize_entry("aud", &self.aud)?;
        map.serialize_entry("exp", &self.exp)?;
        map.serialize_entry("nbf", &self.nbf)?;
        map.serialize_entry("iat", &self.iat)?;
        map.serialize_entry("jti", &self.jti)?;
        map.serialize_entry("scope", &self.scope)?;
        map.serialize_entry("gty", &self.gty)?;
        map.serialize_entry("permissions", &self.permissions)?;

        for custom_claims in &self.custom_claims {
            match custom_claims.value() {
                CustomFieldValue::String(string) => map.serialize_entry(custom_claims.name(), &string),
                CustomFieldValue::Vec(vec) => map.serialize_entry(custom_claims.name(), &vec),
            }?;
        }

        map.end()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum GrantType {
    ClientCredentials,
    AuthorizationCode,
}

impl Display for GrantType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GrantType::ClientCredentials => write!(f, "client_credentials"),
            GrantType::AuthorizationCode => write!(f, "authorization_code"),
        }
    }
}
