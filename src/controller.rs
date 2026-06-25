use std::collections::HashMap;

use actix_web::web::{Data, Either, Form, Json, Path, Query};
use actix_web::{get, post, HttpRequest, HttpResponse};

use crate::model::{
    AppData, AuthorizationCodeTokenRequest, AuthorizeQuery, Claims, ClientCredentialsTokenRequest, GrantType,
    IdTokenClaims, Jwk, Jwks, LoginRequest, LoginResponse, OpenIDMetadata, PermissionsForAudienceRequest, StateQuery,
    TokenRequest, TokenResponse, UpdateCustomClaimsRequest, UpdateUserInfoRequest, UserInfo,
    UserLoginIdentifierRequest, UserLoginPasswordRequest,
};
use crate::store::{AuthorizationData, LoginState};
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

/// OIDC authorization endpoint. Generates a fresh internal `state_token`, binds
/// the caller's parameters to it in the `LoginStatesStore`, and 302s to the
/// HTML login page at `/u/login?state=<state_token>`. The caller's own `state`
/// query parameter (if any) is preserved on the `LoginState` for echo back at
/// `/authorize/resume`.
#[get("/authorize")]
pub async fn authorize(app_data: Data<AppData>, query: Query<AuthorizeQuery>) -> HttpResponse {
    let q = query.into_inner();
    let state_token = uuid::Uuid::new_v4().to_string();
    let login_state = LoginState {
        client_id: q.client_id,
        audience: q.audience,
        redirect_uri: q.redirect_uri,
        scope: q.scope,
        caller_state: q.state,
        nonce: q.nonce,
        username: None,
    };

    if let Err(e) = app_data.login_states().put(state_token.clone(), login_state) {
        return HttpResponse::InternalServerError().body(format!("Failed to create login state: {e}"));
    }

    HttpResponse::Found()
        .append_header(("Location", format!("/u/login?state={state_token}")))
        .finish()
}

/// HTML login form, served at `/u/login?state=<state_token>`. The form posts
/// to `/u/login/password` so browser-driven tests (Playwright) can fill the
/// `input[name="username"]` and `input[name="password"]` selectors and click
/// the primary action button — matching the selectors the ITF `BrowserAuth.java`
/// targets against real Auth0.
#[get("/u/login")]
pub async fn login_page(app_data: Data<AppData>, query: Query<StateQuery>) -> HttpResponse {
    let state = &query.state;

    // Reject if state_token is unknown — prevents the form from rendering for
    // arbitrary URLs.
    match app_data.login_states().get(state) {
        Ok(Some(_)) => {}
        Ok(None) => {
            return HttpResponse::BadRequest()
                .content_type("text/plain")
                .body("Unknown login state");
        }
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("Failed to read login state: {e}"));
        }
    }

    let html = format!(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <title>Sign in</title>
  </head>
  <body>
    <form method="POST" action="/u/login/password?state={state}">
      <label>Email <input name="username" type="email" autocomplete="username" required></label>
      <label>Password <input name="password" type="password" autocomplete="current-password" required></label>
      <input name="state" type="hidden" value="{state}">
      <button type="submit" data-action-button-primary="true">Continue</button>
    </form>
  </body>
</html>"#
    );

    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html)
}

/// First leg of the Auth0 hosted-login dance. Stores the supplied `username`
/// on the `LoginState` keyed by `state_token`, then 302s to the password step.
/// Real Auth0 returns a 302 to `/u/login/password?state=...`; we mirror that.
/// Accepts either JSON or form-urlencoded bodies because the ITF
/// `DashboardAutomation` POSTs JSON while a real browser form would post
/// form-urlencoded.
#[post("/u/login/identifier")]
pub async fn login_identifier(
    app_data: Data<AppData>,
    query: Query<StateQuery>,
    body: Either<Json<UserLoginIdentifierRequest>, Form<UserLoginIdentifierRequest>>,
) -> HttpResponse {
    let req = match body {
        Either::Left(Json(b)) | Either::Right(Form(b)) => b,
    };
    let state_token = &query.state;

    match app_data.login_states().get(state_token) {
        Ok(Some(_)) => {}
        Ok(None) => {
            return HttpResponse::BadRequest()
                .content_type("application/json")
                .body(r#"{"error":"invalid_request","error_description":"Unknown state"}"#);
        }
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("Failed to read login state: {e}"));
        }
    }

    if let Err(e) = app_data.login_states().set_username(state_token, req.username) {
        return HttpResponse::InternalServerError().body(format!("Failed to set username: {e}"));
    }

    HttpResponse::Found()
        .append_header(("Location", format!("/u/login/password?state={state_token}")))
        .finish()
}

/// Second leg of the hosted-login dance. Validates `username` + `password`
/// against the users store, then 302s to `/authorize/resume?state=<state_token>`
/// where the auth code is minted. The identifier step may be skipped (the JVM
/// DashboardAutomation sometimes calls /password directly) — in that case the
/// username from the body is what gets bound to the login state.
#[post("/u/login/password")]
pub async fn login_password(
    app_data: Data<AppData>,
    query: Query<StateQuery>,
    body: Either<Json<UserLoginPasswordRequest>, Form<UserLoginPasswordRequest>>,
) -> HttpResponse {
    let req = match body {
        Either::Left(Json(b)) | Either::Right(Form(b)) => b,
    };
    let state_token = &query.state;

    match app_data.login_states().get(state_token) {
        Ok(Some(_)) => {}
        Ok(None) => {
            return HttpResponse::BadRequest()
                .content_type("application/json")
                .body(r#"{"error":"invalid_request","error_description":"Unknown state"}"#);
        }
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("Failed to read login state: {e}"));
        }
    }

    match app_data.users().authenticate(&req.username, &req.password) {
        Ok(Some(_)) => {}
        Ok(None) => {
            return HttpResponse::Unauthorized()
                .content_type("application/json")
                .body(r#"{"error":"access_denied","error_description":"Invalid credentials"}"#);
        }
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("Failed to authenticate user: {e}"));
        }
    }

    if let Err(e) = app_data.login_states().set_username(state_token, req.username) {
        return HttpResponse::InternalServerError().body(format!("Failed to set username: {e}"));
    }

    HttpResponse::Found()
        .append_header(("Location", format!("/authorize/resume?state={state_token}")))
        .finish()
}

/// Final hop of the Auth0 hosted-login dance. Consumes the `LoginState`
/// (one-shot), mints an authorization code bound to the resolved user, and
/// 302s to the caller's `redirect_uri` with `?code=...&state=...`. The
/// `state` echoed back is the caller's original `state` parameter, not the
/// internal `state_token`.
#[get("/authorize/resume")]
pub async fn authorize_resume(app_data: Data<AppData>, query: Query<StateQuery>) -> HttpResponse {
    let state_token = &query.state;

    let login_state = match app_data.login_states().take(state_token) {
        Ok(Some(s)) => s,
        Ok(None) => {
            return HttpResponse::BadRequest()
                .content_type("application/json")
                .body(r#"{"error":"invalid_request","error_description":"Unknown or already-consumed state"}"#);
        }
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("Failed to read login state: {e}"));
        }
    };

    let username = match login_state.username {
        Some(u) => u,
        None => {
            return HttpResponse::Unauthorized()
                .content_type("application/json")
                .body(r#"{"error":"access_denied","error_description":"Login not completed"}"#);
        }
    };

    let code = uuid::Uuid::new_v4().to_string();
    let data = AuthorizationData {
        audience: login_state.audience,
        user_email: Some(username),
        nonce: login_state.nonce,
    };
    if let Err(e) = app_data.authorizations().put_authorization_data(&code, data) {
        return HttpResponse::InternalServerError().body(format!("Failed to persist authorization: {e}"));
    }

    let sep = if login_state.redirect_uri.contains('?') {
        '&'
    } else {
        '?'
    };
    let caller_state = login_state.caller_state.as_deref().unwrap_or("");
    let location = format!(
        "{}{}code={}&state={}",
        login_state.redirect_uri, sep, code, caller_state
    );

    HttpResponse::Found().append_header(("Location", location)).finish()
}

/// Auth0-shaped `/v2/logout` endpoint. The dashboard JVM builds URLs of the form
/// `<auth0_domain>v2/logout?federated&client_id=<id>&returnTo=<url>` (see
/// `dashboard/dashboard/src/com/wizrocket/dashboard/utils/Auth0Util.java` `getAuth0LogoutURL`).
/// We honour `returnTo` and ignore the rest. If `returnTo` is missing we return a
/// plaintext 200 so the caller does not crash. The query string is parsed manually
/// because real Auth0 callers send bare flags (e.g. `federated`) which the standard
/// serde-urlencoded typed extractor would reject.
#[get("/v2/logout")]
pub async fn logout(req: HttpRequest) -> HttpResponse {
    let return_to = req
        .query_string()
        .split('&')
        .filter_map(|pair| pair.split_once('='))
        .find_map(|(k, v)| if k == "returnTo" { Some(v) } else { None });

    match return_to {
        Some(url) if !url.is_empty() => HttpResponse::Found()
            .append_header(("Location", url.to_string()))
            .finish(),
        _ => HttpResponse::Ok().content_type("text/plain").body("Logged out"),
    }
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
        // client_credentials always uses the singleton user_info — there is no
        // resource-owner identity to resolve.
        let user_info = app_data.user_info().get().expect("Failed to get user info");
        let response: TokenResponse = new_token_response(
            &app_data,
            request.audience.as_str(),
            GrantType::ClientCredentials,
            None,
            user_info,
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

pub async fn jwt_for_authorization_code(
    app_data: Data<AppData>,
    request: AuthorizationCodeTokenRequest,
) -> HttpResponse {
    if request.client_id == CLIENT_ID_VALUE {
        // Client secret is optional for certain clients
        if let Some(expected_secret) = request.client_secret {
            if expected_secret != CLIENT_SECRET_VALUE {
                return HttpResponse::Unauthorized()
                    .content_type("application/json")
                    .body(r#"{"error":"access_denied","error_description":"Unauthorized"}"#);
            }
        }

        // Pull both audience and (if present) the user_email that was bound to the
        // code at `/authorize/resume`. Legacy `/oauth/login` codes have no user
        // attached, in which case we fall through to the singleton user_info.
        let auth_data: AuthorizationData = app_data
            .authorizations()
            .get_data_for_authorization(&request.code)
            .expect("Failed to get authorization data")
            .unwrap_or_default();

        let user_info: UserInfo = auth_data
            .user_email
            .as_deref()
            .and_then(|email| {
                app_data
                    .users()
                    .get(email)
                    .expect("Failed to read users store")
                    .as_ref()
                    .map(UserInfo::from)
            })
            .unwrap_or_else(|| app_data.user_info().get().expect("Failed to get user info"));

        // Prefer the nonce sent on the token-exchange request; fall back to the
        // nonce captured at /authorize.
        let nonce = request.nonce.or(auth_data.nonce);

        let response: TokenResponse = new_token_response(
            &app_data,
            auth_data.audience.as_str(),
            GrantType::AuthorizationCode,
            nonce,
            user_info,
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
    user_info: UserInfo,
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

    let id_token_claims: IdTokenClaims =
        IdTokenClaims::new(app_data.issuer(), CLIENT_ID_VALUE.to_string(), user_info, nonce);

    let random_jwk: Jwk = app_data.jwks().random_jwk().expect("Failed to get JWK");
    let access_token: String = random_jwk.encode(&claims).expect("Failed to generate JWT");
    let id_token: String = random_jwk.encode(&id_token_claims).expect("Failed to generate IdToken");

    TokenResponse::new(access_token, id_token, None)
}

#[cfg(test)]
mod test {
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
            { name = "at_custom_claims_str", value = { String = "str" } },
            { name = "https://clevertap.com/app_metadata", value = { Object = { regions = ["us"], accountMFA = false } } }
        ]

        "#;
        let config: Config = toml::from_str(config_string).unwrap();
        let app_data = AppData::new(&config).unwrap();
        let audience = "audience2";
        let grant_type = GrantType::AuthorizationCode;
        let nonce = Some("nonce".to_string());

        let user_info = app_data.user_info().get().unwrap();
        let token_response = new_token_response(&app_data, audience, grant_type, nonce, user_info);

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
        assert_eq!(
            claims_json.get("https://clevertap.com/app_metadata").unwrap(),
            &json!({ "regions": ["us"], "accountMFA": false })
        );

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
        use crate::model::{ClientCredentialsTokenRequest, TokenRequest, TokenResponse};
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
        use crate::model::{ClientCredentialsTokenRequest, TokenRequest, TokenResponse};
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

    #[actix_web::test]
    async fn logout_redirects_to_return_to_when_present() {
        use super::logout;
        use actix_web::{test, App};

        let app = test::init_service(App::new().service(logout)).await;

        // Mimics the exact shape Auth0Util.getAuth0LogoutURL builds: bare `federated`
        // flag, plus client_id and returnTo. The handler must tolerate the bare flag.
        let req = test::TestRequest::get()
            .uri("/v2/logout?federated&client_id=client_id&returnTo=https://example.com/login.html")
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), 302);
        let location = resp.headers().get("Location").unwrap().to_str().unwrap();
        assert_eq!(location, "https://example.com/login.html");
    }

    #[actix_web::test]
    async fn logout_returns_ok_when_no_return_to() {
        use super::logout;
        use actix_web::{test, App};

        let app = test::init_service(App::new().service(logout)).await;

        let req = test::TestRequest::get().uri("/v2/logout").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), 200);
    }

    #[actix_web::test]
    async fn logout_returns_ok_when_return_to_is_empty() {
        use super::logout;
        use actix_web::{test, App};

        let app = test::init_service(App::new().service(logout)).await;

        let req = test::TestRequest::get().uri("/v2/logout?returnTo=").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), 200);
    }

    /// End-to-end: `/authorize` → `/u/login/identifier` → `/u/login/password` →
    /// `/authorize/resume` → `/oauth/token` (grant_type=authorization_code).
    /// This is the exact sequence the ITF `DashboardAutomation` Branch D drives
    /// against real Auth0. Verifies that the resulting id_token carries the
    /// `admin@clevertap.com` user's identity (not the singleton user_info).
    #[actix_web::test]
    async fn auth0_login_flow_end_to_end() {
        use super::{authorize, authorize_resume, login_identifier, login_password, token};
        use actix_web::{http::header::ContentType, test, web::Data, App};

        let config_str: &str = r#"
        issuer = "http://localauth0:3000/"

        [user_info]
        subject = "auth0|default"
        email = "default@example.com"

        [[user]]
        email = "admin@clevertap.com"
        password = "Adminp@sswd0"
        subject = "auth0|local-admin"
        name = "admin"
        email_verified = true
        custom_fields = [
          { name = "https://clevertap.com/app_metadata", value = { Object = { regions = ["local"], accountMFA = false } } },
          { name = "https://clevertap.com/connectionName", value = { String = "Username-Password-Authentication" } }
        ]

        [[audience]]
        name = "https://clevertap.com"
        permissions = []
        "#;
        let config: Config = toml::from_str(config_str).unwrap();
        let app = test::init_service(
            App::new()
                .app_data(Data::new(AppData::new(&config).unwrap()))
                .service(authorize)
                .service(login_identifier)
                .service(login_password)
                .service(authorize_resume)
                .service(token),
        )
        .await;

        // Step 1: GET /authorize → 302 to /u/login?state=<state_token>
        let authorize_uri = "/authorize?\
            client_id=client_id&\
            audience=https://clevertap.com&\
            redirect_uri=http://localhost:8080/auth0-callback&\
            scope=openid&\
            response_type=code&\
            state=caller-state-abc&\
            nonce=nonce-xyz";
        let req = test::TestRequest::get().uri(authorize_uri).to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 302);
        let location = resp.headers().get("Location").unwrap().to_str().unwrap();
        assert!(location.starts_with("/u/login?state="), "got {location}");
        let state_token = location.strip_prefix("/u/login?state=").unwrap().to_string();

        // Step 2: POST /u/login/identifier?state=... with the username (action ignored)
        let identifier_body = serde_json::json!({
            "username": "admin@clevertap.com",
            "action": "default",
            "js-available": true,
        });
        let req = test::TestRequest::post()
            .uri(&format!("/u/login/identifier?state={state_token}"))
            .insert_header(ContentType::json())
            .set_payload(serde_json::to_string(&identifier_body).unwrap())
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 302);
        let location = resp.headers().get("Location").unwrap().to_str().unwrap();
        assert_eq!(location, &format!("/u/login/password?state={state_token}"));

        // Step 3: POST /u/login/password?state=... with valid creds
        let password_body = serde_json::json!({
            "username": "admin@clevertap.com",
            "password": "Adminp@sswd0",
            "state": state_token,
            "action": "default",
        });
        let req = test::TestRequest::post()
            .uri(&format!("/u/login/password?state={state_token}"))
            .insert_header(ContentType::json())
            .set_payload(serde_json::to_string(&password_body).unwrap())
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 302);
        let location = resp.headers().get("Location").unwrap().to_str().unwrap();
        assert_eq!(location, &format!("/authorize/resume?state={state_token}"));

        // Step 4: GET /authorize/resume → 302 to redirect_uri?code=...&state=caller-state-abc
        let req = test::TestRequest::get()
            .uri(&format!("/authorize/resume?state={state_token}"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 302);
        let location = resp.headers().get("Location").unwrap().to_str().unwrap();
        assert!(
            location.starts_with("http://localhost:8080/auth0-callback?code="),
            "got {location}"
        );
        assert!(location.ends_with("&state=caller-state-abc"), "got {location}");

        // Extract code from Location URL.
        let code_start = location.find("code=").unwrap() + "code=".len();
        let code_end = location.find("&state=").unwrap();
        let code = &location[code_start..code_end];

        // Step 5: POST /oauth/token grant_type=authorization_code with the code →
        // verify the id_token carries the local-admin user's claims.
        let token_body = serde_json::json!({
            "grant_type": "authorization_code",
            "client_id": "client_id",
            "code": code,
        });
        let req = test::TestRequest::post()
            .uri("/oauth/token")
            .insert_header(ContentType::json())
            .set_payload(serde_json::to_string(&token_body).unwrap())
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success(), "got status {}", resp.status());

        let body = test::read_body(resp).await;
        let token_response: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let id_token = token_response["id_token"].as_str().unwrap();
        let claims = extract_payload(id_token);

        assert_eq!(claims["sub"], "auth0|local-admin");
        assert_eq!(claims["email"], "admin@clevertap.com");
        assert_eq!(claims["name"], "admin");
        assert_eq!(claims["nonce"], "nonce-xyz");
        // The Object custom field flattens to a top-level claim.
        assert_eq!(
            claims["https://clevertap.com/app_metadata"],
            serde_json::json!({ "regions": ["local"], "accountMFA": false })
        );
        assert_eq!(
            claims["https://clevertap.com/connectionName"],
            "Username-Password-Authentication"
        );

        let access_token = token_response["access_token"].as_str().unwrap();
        let access_claims = extract_payload(access_token);
        assert_eq!(access_claims["aud"], "https://clevertap.com");
        assert_eq!(access_claims["gty"], "authorization_code");
    }

    #[actix_web::test]
    async fn auth0_login_flow_rejects_bad_password() {
        use super::{authorize, login_password};
        use actix_web::{http::header::ContentType, test, web::Data, App};

        let config_str: &str = r#"
        issuer = "http://localauth0:3000/"

        [[user]]
        email = "admin@clevertap.com"
        password = "Adminp@sswd0"

        [[audience]]
        name = "https://clevertap.com"
        permissions = []
        "#;
        let config: Config = toml::from_str(config_str).unwrap();
        let app = test::init_service(
            App::new()
                .app_data(Data::new(AppData::new(&config).unwrap()))
                .service(authorize)
                .service(login_password),
        )
        .await;

        // /authorize → state
        let req = test::TestRequest::get()
            .uri("/authorize?client_id=client_id&audience=https://clevertap.com&redirect_uri=http://localhost:8080/auth0-callback")
            .to_request();
        let resp = test::call_service(&app, req).await;
        let location = resp.headers().get("Location").unwrap().to_str().unwrap();
        let state_token = location.strip_prefix("/u/login?state=").unwrap().to_string();

        // wrong password → 401
        let body = serde_json::json!({
            "username": "admin@clevertap.com",
            "password": "wrong",
        });
        let req = test::TestRequest::post()
            .uri(&format!("/u/login/password?state={state_token}"))
            .insert_header(ContentType::json())
            .set_payload(serde_json::to_string(&body).unwrap())
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 401);
    }

    #[actix_web::test]
    async fn authorize_resume_rejects_unknown_state() {
        use super::authorize_resume;
        use actix_web::{test, web::Data, App};

        let config = Config::default();
        let app = test::init_service(
            App::new()
                .app_data(Data::new(AppData::new(&config).unwrap()))
                .service(authorize_resume),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/authorize/resume?state=does-not-exist")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 400);
    }
}
