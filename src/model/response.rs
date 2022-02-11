use serde::Serialize;

use crate::BEARER;

#[derive(Serialize)]
pub struct TokenResponse {
    access_token: String,
    scope: String,
    expires_in: i32,
    token_type: String,
}

impl TokenResponse {
    pub fn new(access_token: String, scope_opt: Option<String>) -> Self {
        Self {
            access_token,
            scope: scope_opt.unwrap_or_default(),
            expires_in: 86400,
            token_type: BEARER.to_string(),
        }
    }
}
