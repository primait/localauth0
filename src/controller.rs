use std::collections::HashMap;

use actix_web::web::{Data, Form, Json, Path};
use actix_web::{get, post, HttpResponse};

use crate::model::{
    AppData, AuthorizationCodeTokenRequest, Claims, ClientCredentialsTokenRequest, GrantType, Jwk, Jwks, LoginRequest,
    LoginResponse, PermissionsForAudienceRequest, TokenRequest, TokenResponse, UserInfo,
};
use crate::{CLIENT_ID_VALUE, CLIENT_SECRET_VALUE};

/// .well-known/jwks.json route. This is the standard route exposed by authorities to fetch jwks
#[get("/.well-known/jwks.json")]
pub async fn jwks(app_data: Data<AppData>) -> HttpResponse {
    let jwks: Jwks = app_data.jwks().get().expect("Failed to read JWKS");
    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&jwks).expect("Failed to serialize JWKS to json"))
}

/// Handler for application/json encoded post bodies to the token endpoint
pub async fn jwt_json_body_handler(app_data: Data<AppData>, token_request: Json<TokenRequest>) -> HttpResponse {
    jwt(app_data, token_request.0).await
}

/// Handler for application/x-www-form-urlencoded encoded post bodies to the token endpoint.
/// This is the required format specified by `https://www.rfc-editor.org/rfc/rfc6749#section-4.4.2`. and auth0
pub async fn jwt_form_body_handler(app_data: Data<AppData>, token_request: Form<TokenRequest>) -> HttpResponse {
    jwt(app_data, token_request.0).await
}

/// Generate a new jwt token for a given audience. For `client_credentials` the audience is found in the post body
/// and for `authorization_code` the audience is found in the authorizations cache.
/// All the permissions found in the local store will be included in the generated token.
async fn jwt(app_data: Data<AppData>, token_request: TokenRequest) -> HttpResponse {
    match token_request {
        TokenRequest::ClientCredentials(request) => jwt_for_client_credentials(app_data, request).await,
        TokenRequest::AuthorizationCode(request) => jwt_for_authorization_code(app_data, request).await,
    }
}

/// Logs the "user" in and returns an auth code which can be exchanged for a token
#[post("/oauth/login")]
pub async fn login(app_data: Data<AppData>, login_request: Json<LoginRequest>) -> HttpResponse {
    let code = uuid::Uuid::new_v4().to_string();
    app_data
        .authorizations()
        .put_authorization(&code, login_request.0.audience)
        .expect("Failed to insert authorization");

    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&LoginResponse { code }).expect("Failed to serialize login response to json"))
}

/// List all audience-permissions mappings present in local implementation
#[get("/permissions")]
pub async fn get_permissions(app_data: Data<AppData>) -> HttpResponse {
    let all_audiences: HashMap<String, Vec<String>> =
        app_data.audiences().all().expect("Failed to get inner audiences cache");

    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&all_audiences).expect("Failed to serialize permissions list"))
}

/// Set the given list of permissions for a given audience
#[post("/permissions")]
pub async fn set_permissions_for_audience(
    app_data: Data<AppData>,
    permissions_for_audience_request: Json<PermissionsForAudienceRequest>,
) -> HttpResponse {
    app_data
        .audiences()
        .put_permissions(
            &permissions_for_audience_request.0.audience,
            permissions_for_audience_request.0.permissions,
        )
        .expect("Failed to put permissions for given audience");

    let all_audiences: HashMap<String, Vec<String>> =
        app_data.audiences().all().expect("Failed to get inner audiences cache");

    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&all_audiences).expect("Failed to serialize audiences map"))
}

/// List all permissions set for a given audience
#[get("/permissions/{audience}")]
pub async fn get_permissions_by_audience(app_data: Data<AppData>, audience: Path<String>) -> HttpResponse {
    let audience: String = audience.into_inner();
    let permissions: Vec<String> = app_data
        .audiences()
        .get_permissions(audience.as_str())
        .expect("Failed to get permissions by given audience");

    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&permissions).expect("Failed to serialize permissions list"))
}

/// Remove one jwk and generate new one
#[get("/rotate")]
pub async fn rotate_keys(app_data: Data<AppData>) -> HttpResponse {
    app_data.jwks().rotate_keys().expect("Failed to rotate keys");
    HttpResponse::Ok().content_type("text/plain").body("ok")
}

/// Revoke all jwks keys and generate 3 new jwks
#[get("/revoke")]
pub async fn revoke_keys(app_data: Data<AppData>) -> HttpResponse {
    app_data.jwks().revoke_keys().expect("Failed to revoke keys");
    HttpResponse::Ok().content_type("text/plain").body("ok")
}

pub async fn jwt_for_client_credentials(
    app_data: Data<AppData>,
    request: ClientCredentialsTokenRequest,
) -> HttpResponse {
    if request.client_id == CLIENT_ID_VALUE && request.client_secret == CLIENT_SECRET_VALUE {
        let response: TokenResponse =
            new_token_response(&*app_data, request.audience.as_str(), GrantType::ClientCredentials);

        HttpResponse::Ok()
            .content_type("application/json")
            .body(serde_json::to_string(&response).expect("Failed to serialize TokenResponse"))
    } else {
        HttpResponse::Unauthorized()
            .content_type("application/json")
            .body(r#"{"error":"access_denied","error_description":"Unauthorized"}"#)
    }
}

pub async fn jwt_for_authorization_code(
    app_data: Data<AppData>,
    request: AuthorizationCodeTokenRequest,
) -> HttpResponse {
    if request.client_id == CLIENT_ID_VALUE && request.client_secret == CLIENT_SECRET_VALUE {
        let audience: String = app_data
            .authorizations()
            .get_audience_for_authorization(&request.code)
            .expect("Failed to get audience for authorization");

        let response: TokenResponse = new_token_response(&*app_data, audience.as_str(), GrantType::AuthorizationCode);

        HttpResponse::Ok()
            .content_type("application/json")
            .body(serde_json::to_string(&response).expect("Failed to serialize TokenResponse"))
    } else {
        HttpResponse::Unauthorized()
            .content_type("application/json")
            .body(r#"{"error":"access_denied","error_description":"Unauthorized"}"#)
    }
}

fn new_token_response(app_data: &AppData, audience: &str, grant_type: GrantType) -> TokenResponse {
    let permissions: Vec<String> = app_data
        .audiences()
        .get_permissions(audience)
        .expect("Failed to get permissions");

    let claims: Claims = Claims::new(
        audience.to_string(),
        permissions,
        app_data.config().issuer().to_string(),
        grant_type,
    );

    let user_info: UserInfo = UserInfo::from_config(app_data.config(), audience.to_string());

    let random_jwk: Jwk = app_data.jwks().random_jwk().expect("Failed to get JWK");
    let access_token: String = random_jwk.encode(&claims).expect("Failed to generate JWT");
    let id_token: String = random_jwk.encode(&user_info).expect("Failed to generate IdToken");

    TokenResponse::new(access_token, id_token, None)
}
