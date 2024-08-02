use super::{Issuer, Jwk, TokenResponse};
use crate::controller;
use serde::Serialize;

#[derive(Serialize)]
pub struct OpenIDMetadata {
    issuer: Issuer,
    authorization_endpoint: String,
    token_endpoint: String,
    jwks_uri: String,
    response_types_supported: Vec<String>,
    subject_types_supported: Vec<String>,
    id_token_signing_alg_values_supported: Vec<String>,
}

fn endpoint_to_url(base_uri: &str, endpoint: &str) -> String {
    format!("{base_uri}{endpoint}")
}

impl OpenIDMetadata {
    pub fn new(
        issuer: &Issuer,
        random_jwk: &Jwk,
        // The base uri for, concatenated with endpoints to generate the urls
        base_uri: &str,
    ) -> Self {
        let authorization_endpoint = endpoint_to_url(base_uri, controller::login::ENDPOINT);
        let token_endpoint = endpoint_to_url(base_uri, controller::token::ENDPOINT);
        let jwks_uri = endpoint_to_url(base_uri, controller::jwks::ENDPOINT);

        Self {
            issuer: issuer.clone(),
            authorization_endpoint,
            token_endpoint,
            jwks_uri,
            response_types_supported: TokenResponse::response_types_supported(),
            subject_types_supported: vec!["public".to_string()],
            id_token_signing_alg_values_supported: vec![random_jwk.alg().to_string()],
        }
    }
}
