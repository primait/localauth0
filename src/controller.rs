use std::collections::HashMap;

use actix_web::web::{Data, Either, Form, Json, Path};
use actix_web::{get, post, HttpRequest, HttpResponse};

use crate::model::{
    AppData, AuthorizationCodeTokenRequest, Claims, ClientCredentialsTokenRequest, GrantType, IdTokenClaims, Jwk, Jwks,
    LoginRequest, LoginResponse, OpenIDMetadata, PermissionsForAudienceRequest, TokenRequest, TokenResponse,
    UpdateCustomClaimsRequest, UpdateUserInfoRequest,
};
use crate::{CLIENT_ID_VALUE, CLIENT_SECRET_VALUE};

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

#[get("/oauth/token/custom_claims")]
pub async fn get_custom_claims(app_data: Data<AppData>) -> HttpResponse {
    let custom_claims = app_data.custom_claims().all().expect("Failed to get custom claims");
    HttpResponse::Ok().json(custom_claims)
}

#[post("/oauth/token/custom_claims")]
pub async fn set_custom_claims(
    app_data: Data<AppData>,
    update_custom_claims_request: Json<UpdateCustomClaimsRequest>,
) -> HttpResponse {
    app_data
        .custom_claims()
        .put_custom_fields(update_custom_claims_request.into_inner().custom_claims)
        .expect("Failed to update custom_claims");

    HttpResponse::Ok().into()
}

#[get("/oauth/token/user_info")]
pub async fn get_user_info(app_data: Data<AppData>) -> HttpResponse {
    let user_info = app_data.user_info().get().expect("Failed to get user info");
    HttpResponse::Ok().json(user_info)
}

#[post("/oauth/token/user_info")]
pub async fn set_user_info(
    app_data: Data<AppData>,
    update_user_info_request: Json<UpdateUserInfoRequest>,
) -> HttpResponse {
    let user_info = app_data
        .user_info()
        .update(update_user_info_request.into_inner())
        .expect("Failed to update user info");

    HttpResponse::Ok().json(user_info)
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
    let jwk = app_data
        .jwks()
        .random_jwk()
        .expect("No JWKs configured. Cannot get openid configuration");

    let metadata = OpenIDMetadata::new(app_data.issuer(), &jwk, &base_uri);
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

    let custom_claims = app_data
        .custom_claims()
        .all()
        .expect("Failed to get custom claims")
        .to_owned();

    let claims: Claims = Claims::new(
        audience.to_string(),
        permissions,
        app_data.issuer().0.to_string(),
        app_data.subject().0.to_string(),
        grant_type,
        custom_claims,
    );

    let user_info = app_data.user_info().get().expect("Failed to get user info");
    let id_token_claims: IdTokenClaims =
        IdTokenClaims::new(app_data.issuer(), CLIENT_ID_VALUE.to_string(), user_info, nonce);

    let random_jwk: Jwk = app_data.jwks().random_jwk().expect("Failed to get JWK");
    let access_token: String = random_jwk.encode(&claims).expect("Failed to generate JWT");
    let id_token: String = random_jwk.encode(&id_token_claims).expect("Failed to generate IdToken");

    TokenResponse::new(access_token, id_token, None)
}

#[cfg(test)]
mod test {
    use crate::model::{Jwks, TokenResponse};
    use crate::{
        config::Config,
        model::{AppData, GrantType},
        CLIENT_ID_VALUE,
    };
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn new_token_response_should_return_an_access_token_and_an_id_token() {
        use super::new_token_response;

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
        let app_data = AppData::new(&config).unwrap();
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

    #[actix_web::test]
    async fn healthcheck_test() {
        use super::healthcheck;
        use actix_web::{body, test, web::Data, App};

        let config = Config::default();
        let app = test::init_service(
            App::new()
                .app_data(Data::new(AppData::new(&config).unwrap()))
                .service(healthcheck),
        )
        .await;

        let req = test::TestRequest::get().uri("/check").to_request();
        let resp = test::call_service(&app, req).await;
        let status = resp.status();
        let bytes = body::to_bytes(resp.into_body()).await.unwrap();
        let body = String::from_utf8(bytes.to_vec()).unwrap();

        assert!(status.is_success());
        assert_eq!(body, "ok");
    }

    #[actix_web::test]
    async fn well_known_jwks_test() {
        use super::jwks;
        use crate::model::Jwks;
        use actix_web::{test, web::Data, App};

        let config = Config::default();
        let app = test::init_service(
            App::new()
                .app_data(Data::new(AppData::new(&config).unwrap()))
                .service(jwks),
        )
        .await;

        let req = test::TestRequest::get().uri(jwks::ENDPOINT).to_request();
        let resp: Jwks = test::call_and_read_body_json(&app, req).await;
        assert_eq!(resp.keys.len(), 3);
    }

    #[actix_web::test]
    async fn permissions_test() {
        use super::{get_permissions, get_permissions_by_audience, set_permissions_for_audience};
        use crate::model::PermissionsForAudienceRequest;
        use actix_web::{http::header::ContentType, test, web::Data, App};

        let config = Config::default();
        let app = test::init_service(
            App::new()
                .app_data(Data::new(AppData::new(&config).unwrap()))
                .service(get_permissions)
                .service(get_permissions_by_audience)
                .service(set_permissions_for_audience),
        )
        .await;

        let req = test::TestRequest::get().uri("/permissions").to_request();
        let resp: HashMap<String, Vec<String>> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(resp.keys().len(), 0);

        let req = test::TestRequest::get().uri("/permissions/hello").to_request();
        let resp: Vec<String> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(resp.len(), 0);

        let payload = PermissionsForAudienceRequest {
            audience: "hello".to_string(),
            permissions: vec!["world".to_string()],
        };

        let req = test::TestRequest::post()
            .uri("/permissions")
            .insert_header(ContentType::json())
            .set_payload(serde_json::to_string(&payload).unwrap())
            .to_request();

        let resp: HashMap<String, Vec<String>> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(resp.len(), 1);
        let first = resp.get("hello").unwrap();
        assert_eq!(first, &vec!["world"]);

        let req = test::TestRequest::get().uri("/permissions/hello").to_request();
        let resp: Vec<String> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(resp, vec!["world"]);
    }

    #[actix_web::test]
    async fn custom_claims_test() {
        use super::{get_custom_claims, set_custom_claims, token};
        use crate::config::CustomField;
        use crate::model::{Claims, ClientCredentialsTokenRequest, TokenRequest, TokenResponse, UserInfo};
        use actix_web::{http::header::ContentType, test, web::Data, App};

        let path = "/oauth/token/custom_claims";

        let config = Config::default();
        let app = test::init_service(
            App::new()
                .app_data(Data::new(AppData::new(&config).unwrap()))
                .service(get_custom_claims)
                .service(set_custom_claims)
                .service(token),
        )
        .await;

        let req = test::TestRequest::get().uri(path).to_request();
        let resp: Vec<CustomField> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(resp.len(), 0);

        let payload = json!({
            "custom_claims": [
                {
                    "name": "custom_claim1",
                    "value": {
                        "String": "value1"
                    }
                },
                {
                    "name": "custom_claim2",
                    "value": {
                        "Vec": ["val1", "val2"]
                    }
                }
            ]
        });

        let req = test::TestRequest::post()
            .uri(path)
            .insert_header(ContentType::json())
            .set_payload(serde_json::to_string(&payload).unwrap())
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let req = test::TestRequest::get().uri(path).to_request();
        let resp: Vec<CustomField> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(resp.len(), 2);
        assert_eq!(resp[0].name(), "custom_claim1");
        assert_eq!(
            serde_json::to_string(resp[0].value()).unwrap(),
            "{\"String\":\"value1\"}"
        );
        assert_eq!(resp[1].name(), "custom_claim2");
        assert_eq!(
            serde_json::to_string(resp[1].value()).unwrap(),
            "{\"Vec\":[\"val1\",\"val2\"]}"
        );

        let get_token_request = TokenRequest::ClientCredentials(ClientCredentialsTokenRequest {
            client_id: "client_id".to_string(),
            client_secret: "client_secret".to_string(),
            audience: "test_audience".to_string(),
        });

        let req = test::TestRequest::post()
            .uri("/oauth/token")
            .insert_header(ContentType::json())
            .set_payload(serde_json::to_string(&get_token_request).unwrap())
            .to_request();
        let resp: TokenResponse = test::call_and_read_body_json(&app, req).await;

        let claims = extract_payload(resp.access_token());
        assert_eq!(claims["custom_claim1"], "value1");
        assert_eq!(claims["custom_claim2"], json!(["val1", "val2"]));
    }

    #[actix_web::test]
    async fn user_info_test() {
        use super::{get_user_info, set_user_info, token};
        use crate::config::UserInfoConfig;
        use crate::model::UserInfo;
        use crate::model::{Claims, ClientCredentialsTokenRequest, TokenRequest, TokenResponse};
        use actix_web::{http::header::ContentType, test, web::Data, App};

        let path = "/oauth/token/user_info";
        let default_user_info = UserInfoConfig::default();

        let config = Config::default();
        let app = test::init_service(
            App::new()
                .app_data(Data::new(AppData::new(&config).unwrap()))
                .service(get_user_info)
                .service(set_user_info)
                .service(token),
        )
        .await;

        let req = test::TestRequest::get().uri(path).to_request();
        let resp: UserInfo = test::call_and_read_body_json(&app, req).await;
        assert_eq!(resp, (&default_user_info).into());

        let payload = json!({
            "subject": "subject",
            "name": "name",
            "given_name": "given_name",
            "family_name": "family_name",
            "nickname": "nickname",
            "locale": "locale",
            "gender": "gender",
            "birthdate": "birthdate",
            "email": "email",
            "email_verified": false,
            "picture": "picture",
            "custom_fields": [{
                "name": "custom_field1",
                "value": {
                    "String": "value1"
                }
            },{
                "name": "custom_field2",
                "value": {
                    "String": "value2"
                }
            }],
        });

        let req = test::TestRequest::post()
            .uri(path)
            .insert_header(ContentType::json())
            .set_payload(serde_json::to_string(&payload).unwrap())
            .to_request();

        let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;

        assert_eq!(resp["sub"], payload["subject"]);
        assert_eq!(resp["name"], payload["name"]);
        assert_eq!(resp["given_name"], payload["given_name"]);
        assert_eq!(resp["family_name"], payload["family_name"]);
        assert_eq!(resp["nickname"], payload["nickname"]);
        assert_eq!(resp["locale"], payload["locale"]);
        assert_eq!(resp["gender"], payload["gender"]);
        assert_eq!(resp["birthdate"], payload["birthdate"]);
        assert_eq!(resp["email"], payload["email"]);
        assert_eq!(resp["email_verified"], payload["email_verified"]);
        assert_eq!(resp["picture"], payload["picture"]);
        assert_eq!(resp["custom_field1"], "value1");
        assert_eq!(resp["custom_field2"], "value2");

        let get_token_request = TokenRequest::ClientCredentials(ClientCredentialsTokenRequest {
            client_id: "client_id".to_string(),
            client_secret: "client_secret".to_string(),
            audience: "test_audience".to_string(),
        });

        let req = test::TestRequest::post()
            .uri("/oauth/token")
            .insert_header(ContentType::json())
            .set_payload(serde_json::to_string(&get_token_request).unwrap())
            .to_request();
        let resp: TokenResponse = test::call_and_read_body_json(&app, req).await;

        let user_info = extract_payload(resp.id_token());
        assert_eq!(user_info["sub"], payload["subject"]);
        assert_eq!(user_info["name"], payload["name"]);
        assert_eq!(user_info["given_name"], payload["given_name"]);
        assert_eq!(user_info["family_name"], payload["family_name"]);
        assert_eq!(user_info["nickname"], payload["nickname"]);
        assert_eq!(user_info["locale"], payload["locale"]);
        assert_eq!(user_info["gender"], payload["gender"]);
        assert_eq!(user_info["birthdate"], payload["birthdate"]);
        assert_eq!(user_info["email"], payload["email"]);
        assert_eq!(user_info["email_verified"], payload["email_verified"]);
        assert_eq!(user_info["picture"], payload["picture"]);
        assert_eq!(user_info["custom_field1"], "value1");
        assert_eq!(user_info["custom_field2"], "value2");
    }

    fn extract_payload(token: &str) -> serde_json::Value {
        let parts: Vec<&str> = token.split('.').collect();
        let v = URL_SAFE_NO_PAD.decode(parts[1]).unwrap();
        serde_json::from_slice(&v).unwrap()
    }
}
