use actix_web::web::{Data, Json};
use actix_web::{get, post, HttpResponse};
use std::collections::HashMap;

use crate::model::{AppData, Claims, Jwk, Jwks, PermissionsForAudienceRequest, TokenRequest, TokenResponse};
use crate::{CLIENT_ID_VALUE, CLIENT_SECRET_VALUE};

#[get("/.well-known/jwks.json")]
pub async fn jwks(app_data: Data<AppData>) -> HttpResponse {
    let jwks: Jwks = app_data.jwks_store().get().expect("Failed to read JWKS");
    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&jwks).expect("Failed to serialize JWKS to json"))
}

#[post("/oauth/token")]
pub async fn jwt(app_data: Data<AppData>, token_request: Json<TokenRequest>) -> HttpResponse {
    let request: TokenRequest = token_request.0;

    if request.client_id == CLIENT_ID_VALUE && request.client_secret == CLIENT_SECRET_VALUE {
        let permissions: Vec<String> = app_data
            .audience()
            .get_permissions(&request.audience)
            .expect("Failed to get permissions");

        let claims: Claims = Claims::new(request.audience, permissions);
        let random_jwk: Jwk = app_data.jwks_store().get_random().expect("Failed to get JWK");
        let access_token: String = claims.to_string(&random_jwk).expect("Failed to generate JWT");
        let response: TokenResponse = TokenResponse::new(access_token, None);

        HttpResponse::Ok()
            .content_type("application/json")
            .body(serde_json::to_string(&response).expect("Failed to serialize TokenResponse"))
    } else {
        HttpResponse::Unauthorized()
            .content_type("application/json")
            .body(r#"{"error":"access_denied","error_description":"Unauthorized"}"#)
    }
}

#[post("/permissions")]
pub async fn set_permissions_for_audience(
    app_data: Data<AppData>,
    permissions_for_audience_request: Json<PermissionsForAudienceRequest>,
) -> HttpResponse {
    app_data
        .audience()
        .set_permissions(
            &permissions_for_audience_request.0.audience,
            permissions_for_audience_request.0.permissions,
        )
        .expect("Failed to put permissions for given audience");

    let all_audiences: HashMap<String, Vec<String>> =
        app_data.audience().all().expect("Failed to get inner audiences cache");

    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&all_audiences).expect("Failed to serialize audiences map"))
}
