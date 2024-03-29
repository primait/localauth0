use std::collections::HashMap;

use actix_web::web::{Data, Either, Form, Json, Path};
use actix_web::{get, post, HttpRequest, HttpResponse};

use crate::model::{
    AppData, AuthorizationCodeTokenRequest, Claims, ClientCredentialsTokenRequest, GrantType, IdTokenClaims, Jwk, Jwks,
    LoginRequest, LoginResponse, OpenIDMetadata, PermissionsForAudienceRequest, TokenRequest, TokenResponse,
};
use crate::{CLIENT_ID_VALUE, CLIENT_SECRET_VALUE};
///
/// Remove one jwk and generate new one
#[get("/check")]
pub async fn healthcheck() -> HttpResponse {
    HttpResponse::Ok().content_type("text/plain").body("ok")
}

/// .well-known/jwks.json route. This is the standard route exposed by authorities to fetch jwks
#[get("/.well-known/jwks.json")]
pub async fn jwks(app_data: Data<AppData>) -> HttpResponse {
    let jwks: Jwks = app_data.jwks().get().expect("Failed to read JWKS");
    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&jwks).expect("Failed to serialize JWKS to json"))
}

impl jwks {
    pub const ENDPOINT: &'static str = "/.well-known/jwks.json";
}

/// Generate a new jwt token for a given audience. For `client_credentials` the audience is found in the post body
/// and for `authorization_code` the audience is found in the authorizations cache.
/// All the permissions found in the local store will be included in the generated token.
#[post("/oauth/token")]
async fn token(app_data: Data<AppData>, token_request: Either<Json<TokenRequest>, Form<TokenRequest>>) -> HttpResponse {
    let (Either::Left(Json(token_request)) | Either::Right(Form(token_request))) = token_request;

    match token_request {
        TokenRequest::ClientCredentials(request) => jwt_for_client_credentials(app_data, request).await,
        TokenRequest::AuthorizationCode(request) => jwt_for_authorization_code(app_data, request).await,
    }
}

impl token {
    pub const ENDPOINT: &'static str = "/oauth/token";
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
impl login {
    pub const ENDPOINT: &'static str = "/oauth/login";
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

/// .well-known/jwks.json route. This is the standard route to fetch the openid configuration
/// See <https://openid.net/specs/openid-connect-discovery-1_0.html#WellKnownRegistry>
#[get("/.well-known/openid-configuration")]
pub async fn openid_configuration(app_data: Data<AppData>, req: HttpRequest) -> HttpResponse {
    let conn = req.connection_info();
    let base_uri = format!("{}://{}", conn.scheme(), conn.host());

    let metadata = OpenIDMetadata::new(app_data.jwks(), app_data.config(), &base_uri);
    HttpResponse::Ok().json(&metadata)
}

pub async fn jwt_for_client_credentials(
    app_data: Data<AppData>,
    request: ClientCredentialsTokenRequest,
) -> HttpResponse {
    if request.client_id == CLIENT_ID_VALUE && request.client_secret == CLIENT_SECRET_VALUE {
        let response: TokenResponse =
            new_token_response(&app_data, request.audience.as_str(), GrantType::ClientCredentials, None);

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

        let response: TokenResponse = new_token_response(
            &app_data,
            audience.as_str(),
            GrantType::AuthorizationCode,
            request.nonce,
        );

        HttpResponse::Ok()
            .content_type("application/json")
            .body(serde_json::to_string(&response).expect("Failed to serialize TokenResponse"))
    } else {
        HttpResponse::Unauthorized()
            .content_type("application/json")
            .body(r#"{"error":"access_denied","error_description":"Unauthorized"}"#)
    }
}

fn new_token_response(
    app_data: &AppData,
    audience: &str,
    grant_type: GrantType,
    nonce: Option<String>,
) -> TokenResponse {
    let permissions: Vec<String> = app_data
        .audiences()
        .get_permissions(audience)
        .expect("Failed to get permissions");

    let custom_claims = app_data.config().access_token().custom_claims().to_owned();
    let claims: Claims = Claims::new(
        audience.to_string(),
        permissions,
        app_data.config().issuer().to_string(),
        app_data.config().subject().to_string(),
        grant_type,
        custom_claims,
    );

    let id_token_claims: IdTokenClaims = IdTokenClaims::new(app_data.config(), CLIENT_ID_VALUE.to_string(), nonce);

    let random_jwk: Jwk = app_data.jwks().random_jwk().expect("Failed to get JWK");
    let access_token: String = random_jwk.encode(&claims).expect("Failed to generate JWT");
    let id_token: String = random_jwk.encode(&id_token_claims).expect("Failed to generate IdToken");

    TokenResponse::new(access_token, id_token, None)
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use crate::{
        config::Config,
        model::{AppData, GrantType},
        CLIENT_ID_VALUE,
    };

    use super::new_token_response;

    #[test]
    fn new_token_response_should_return_an_access_token_and_an_id_token() {
        let config_string: &str = r#"
        issuer = "https://prima.localauth0.com/"

        [user_info]
        subject = "google-apps|developers@prima.it"
        name = "Local"
        given_name = "Locie"
        family_name = "Auth0"
        nickname = "locie.auth0"
        locale = "en"
        gender = "none"
        birthdate = "2022-02-11"
        email = "developers@prima.it"
        email_verified = true
        picture = "https://github.com/primait/localauth0/blob/6f71c9318250219a9d03fb72afe4308b8824aef7/web/assets/static/media/localauth0.png"
        custom_fields = [
            { name = "roles", value = { Vec = ["fake:auth"] } }
        ]

        [[audience]]
        name = "audience1"
        permissions = ["audience1:permission1", "audience1:permission2"]

        [[audience]]
        name = "audience2"
        permissions = ["audience2:permission1"]

        [access_token]
        custom_claims = [
            { name = "at_custom_claims_str", value = { String = "str" } }
        ]
        
        "#;
        let config: Config = toml::from_str(config_string).unwrap();
        let app_data = AppData::new(config).unwrap();
        let audience = "audience2";
        let grant_type = GrantType::AuthorizationCode;
        let nonce = Some("nonce".to_string());

        let token_response = new_token_response(&app_data, audience, grant_type, nonce);

        let access_token = token_response.access_token();
        let jwks = app_data.jwks().get().unwrap();
        let claims_json: serde_json::Value = jwks
            .parse(access_token, &[audience])
            .expect("failed to parse access_token");

        assert_eq!(claims_json.get("aud").unwrap(), audience);
        assert!(claims_json.get("iat").is_some());
        assert!(claims_json.get("exp").is_some());
        assert!(claims_json.get("scope").is_some());
        assert_eq!(claims_json.get("iss").unwrap(), "https://prima.localauth0.com/");
        assert_eq!(claims_json.get("gty").unwrap(), "authorization_code");
        assert_eq!(
            claims_json.get("permissions").unwrap(),
            &json!(["audience2:permission1"])
        );

        assert_eq!(claims_json.get("at_custom_claims_str").unwrap(), "str");

        let id_token = token_response.id_token();

        let jwks = app_data.jwks().get().unwrap();
        let claims_json: serde_json::Value = jwks
            .parse(id_token, &[CLIENT_ID_VALUE])
            .expect("failed to parse id_token");

        assert_eq!(claims_json.get("sub").unwrap(), "google-apps|developers@prima.it");
        assert_eq!(claims_json.get("aud").unwrap(), CLIENT_ID_VALUE);
        assert!(claims_json.get("iat").is_some());
        assert!(claims_json.get("exp").is_some());
        assert_eq!(claims_json.get("iss").unwrap(), "https://prima.localauth0.com/");
        assert_eq!(claims_json.get("name").unwrap(), "Local");
        assert_eq!(claims_json.get("given_name").unwrap(), "Locie");
        assert_eq!(claims_json.get("family_name").unwrap(), "Auth0");
        assert_eq!(claims_json.get("nickname").unwrap(), "locie.auth0");
        assert_eq!(claims_json.get("locale").unwrap(), "en");
        assert_eq!(claims_json.get("gender").unwrap(), "none");
        assert_eq!(claims_json.get("birthdate").unwrap(), "2022-02-11");
        assert_eq!(claims_json.get("email").unwrap(), "developers@prima.it");
        assert_eq!(claims_json.get("email_verified").unwrap(), true);
        assert_eq!(claims_json.get("picture").unwrap(), "https://github.com/primait/localauth0/blob/6f71c9318250219a9d03fb72afe4308b8824aef7/web/assets/static/media/localauth0.png");
        assert_eq!(claims_json.get("nonce").unwrap(), "nonce");
    }
}
