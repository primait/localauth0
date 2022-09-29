use std::fmt::{Display, Formatter};

use serde::{
    de, Deserialize, Serialize, Serializer,
    __private::fmt,
    de::{MapAccess, Visitor},
    ser::SerializeMap,
};

use crate::config::{CustomField, CustomFieldValue};

#[derive(Debug)]
pub struct Claims {
    aud: String,
    iat: Option<i64>,
    exp: Option<i64>,
    scope: String,
    iss: String,
    gty: GrantType,
    //#[serde(default)]
    permissions: Vec<String>,
    //#[serde(default)]
    custom_claims: Vec<CustomField>,
}

impl Claims {
    pub fn new(
        aud: String,
        permissions: Vec<String>,
        iss: String,
        gty: GrantType,
        custom_claims: Vec<CustomField>,
    ) -> Self {
        Self {
            aud,
            iat: Some(chrono::Utc::now().timestamp()),
            exp: Some(chrono::Utc::now().timestamp() + 60000),
            scope: permissions.join(" "),
            iss,
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

    pub fn grant_type(&self) -> &GrantType {
        &self.gty
    }

    #[cfg(test)]
    pub fn custom_claims(&self) -> &Vec<CustomField> {
        &&self.custom_claims
    }
}

impl<'de> Deserialize<'de> for Claims {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ClaimsVisitor;

        // #[derive(Deserialize)]
        // #[serde(field_identifier)]
        enum Field {
            Aud,
            Iat,
            Exp,
            Scope,
            Iss,
            Gty,
            Permissions,
            CustomClaims(String),
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("claims fields")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "aud" => Ok(Field::Aud),
                            "iat" => Ok(Field::Iat),
                            "exp" => Ok(Field::Exp),
                            "scope" => Ok(Field::Scope),
                            "iss" => Ok(Field::Iss),
                            "gty" => Ok(Field::Gty),
                            "permissions" => Ok(Field::Permissions),
                            _ => Ok(Field::CustomClaims(value.to_string())),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        impl<'de> Visitor<'de> for ClaimsVisitor {
            type Value = Claims;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Claims")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Claims, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut aud = None;
                let mut iat = None;
                let mut exp = None;
                let mut scope = None;
                let mut iss = None;
                let mut gty = None;
                let mut permissions = None;
                let mut custom_claims: Vec<CustomField> = vec![];

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Aud => {
                            if aud.is_some() {
                                return Err(de::Error::duplicate_field("aud"));
                            }
                            aud = Some(map.next_value()?);
                        }
                        Field::Iat => {
                            if iat.is_some() {
                                return Err(de::Error::duplicate_field("iat"));
                            }
                            iat = Some(map.next_value()?);
                        }
                        Field::Exp => {
                            if exp.is_some() {
                                return Err(de::Error::duplicate_field("exp"));
                            }
                            exp = Some(map.next_value()?);
                        }
                        Field::Scope => {
                            if scope.is_some() {
                                return Err(de::Error::duplicate_field("scope"));
                            }
                            scope = Some(map.next_value()?);
                        }
                        Field::Iss => {
                            if iss.is_some() {
                                return Err(de::Error::duplicate_field("iss"));
                            }
                            iss = Some(map.next_value()?);
                        }
                        Field::Gty => {
                            if gty.is_some() {
                                return Err(de::Error::duplicate_field("gty"));
                            }
                            gty = Some(map.next_value()?);
                        }
                        Field::Permissions => {
                            if permissions.is_some() {
                                return Err(de::Error::duplicate_field("permissions"));
                            }
                            permissions = Some(map.next_value()?);
                        }
                        Field::CustomClaims(field_name) => {
                            // TODO: check
                            // if custom_claims.contains(&field_name) {
                            //     return Err(de::Error::duplicate_field(&field_name));
                            // }
                            let custom_field =
                                CustomField::new(field_name.to_string(), CustomFieldValue::String(map.next_value()?));
                            custom_claims.push(custom_field);
                        }
                    }
                }
                let aud = aud.ok_or_else(|| de::Error::missing_field("aud"))?;
                let iat = iat.ok_or_else(|| de::Error::missing_field("iat"))?;
                let exp = exp.ok_or_else(|| de::Error::missing_field("exp"))?;
                let scope = scope.ok_or_else(|| de::Error::missing_field("scope"))?;
                let iss = iss.ok_or_else(|| de::Error::missing_field("iss"))?;
                let gty = gty.ok_or_else(|| de::Error::missing_field("gty"))?;
                let permissions: Vec<String> = permissions.unwrap_or_default();
                Ok(Claims {
                    aud,
                    iat,
                    exp,
                    scope,
                    iss,
                    gty,
                    permissions,
                    custom_claims,
                })
            }
        }
        const FIELDS: &[&str] = &[
            "aud",
            "iat",
            "exp",
            "scope",
            "iss",
            "gty",
            "permissions",
            "custom_claims",
        ];
        deserializer.deserialize_struct("Claims", FIELDS, ClaimsVisitor)
    }
}

impl Serialize for Claims {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Should be serialized as map being that keys should be statically defined to serialize as
        // struct
        let mut map = serializer.serialize_map(None)?;

        map.serialize_entry("aud", &self.aud)?;
        map.serialize_entry("iat", &self.iat)?;
        map.serialize_entry("exp", &self.exp)?;
        map.serialize_entry("scope", &self.scope)?;
        map.serialize_entry("iss", &self.iss)?;
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
