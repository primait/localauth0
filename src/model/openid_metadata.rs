use crate::config::Config;
use crate::controller;
use serde::Serialize;

use super::{JwksStore, TokenResponse};

#[derive(Serialize)]
pub struct OpenIDMetadata {
    issuer: String,
    authorization_endpoint: String,
    token_endpoint: String,
    jwks_uri: String,
    response_types_supported: Vec<String>,
    subject_types_supported: Vec<String>,
    id_token_signing_alg_values_supported: Vec<String>,
}

fn host_endpoint_to_url(host: &str, endpoint: &str) -> String {
    format!("{host}{endpoint}")
}

impl OpenIDMetadata {
    pub fn new(
        jwks: &JwksStore,
        config: &Config,
        // The host of the server, it will be used to generate the URLs in the returned metadata
        host: &str,
    ) -> Self {
        let authorization_endpoint = host_endpoint_to_url(host, controller::login::ENDPOINT);
        let token_endpoint = host_endpoint_to_url(host, controller::token::ENDPOINT);
        let jwks_uri = host_endpoint_to_url(host, controller::jwks::ENDPOINT);

        let jwk = jwks.random_jwk().expect("Failed to get a jwk");

        Self {
            issuer: config.issuer().clone(),
            authorization_endpoint,
            token_endpoint,
            jwks_uri,
            response_types_supported: TokenResponse::response_types_supported(),
            subject_types_supported: vec!["public".to_string()],
            id_token_signing_alg_values_supported: vec![jwk.alg().to_string()],
        }
    }
}
