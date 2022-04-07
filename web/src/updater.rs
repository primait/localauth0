use web_sys::HtmlInputElement;
use yew::format::{Nothing, Text};
use yew::services::fetch::{Request, Response};
use yew::services::FetchService;

use crate::message::Msg;
use crate::model::{Jwt, Model};

pub fn update(model: &mut Model, message: Msg) -> bool {
    match message {
        Msg::AudienceFocusOut => {
            if let Some(input) = model.audience_input_ref.cast::<HtmlInputElement>() {
                let audience: String = input.value();
                let url: String = format!("/permissions/{}", &audience);
                model.audience = Some(audience);

                let request = get(url.as_str());

                let callback = model.link.callback(|response: Response<Text>| {
                    let body_str: String = response.into_body().unwrap();
                    let body: Option<Vec<String>> = serde_json::from_str(&body_str).unwrap();
                    Msg::ShowPermissions(body.unwrap_or_default().into_iter().collect())
                });

                let task = FetchService::fetch(request, callback).expect("Failed to start request");
                model.task = Some(task);
            }
            true
        }
        Msg::ShowPermissions(permissions) => {
            model.permissions = permissions;
            true
        }
        Msg::GenerateToken => {
            let request = post(
                "/oauth/token",
                &TokenRequest {
                    client_id: "client_id".to_string(),
                    client_secret: "client_secret".to_string(),
                    audience: model.audience.clone().unwrap(),
                    grant_type: "client_credentials".to_string(),
                },
            );

            let callback = model.link.callback(|response: Response<Text>| {
                let body_str: String = response.into_body().unwrap();
                let body: Option<Jwt> = serde_json::from_str(&body_str).unwrap();
                Msg::TokenReceived(body)
            });

            let task = FetchService::fetch(request, callback).expect("Failed to start request");
            model.task = Some(task);
            false
        }
        Msg::TokenReceived(token_opt) => {
            model.token = token_opt;
            true
        }
        Msg::AddPermission => {
            if let Some(input) = model.permission_input_ref.cast::<HtmlInputElement>() {
                let value: String = input.value();
                if !value.is_empty() {
                    model.permissions.insert(input.value());
                    input.set_value("");
                }
            }
            true
        }
        Msg::RemovePermission(permission) => {
            model.permissions = model
                .permissions
                .clone()
                .into_iter()
                .filter(|v| v != &permission)
                .collect();
            true
        }

        Msg::SetPermissions => {
            if let Some(input) = model.audience_input_ref.cast::<HtmlInputElement>() {
                model.audience = Some(input.value());
                let request = post(
                    "/permissions",
                    &PermissionsForAudience {
                        audience: model.audience.clone().unwrap(),
                        permissions: model.permissions.clone().into_iter().collect(),
                    },
                );

                let callback = model.link.callback(|_response: Response<Text>| Msg::GenerateToken);
                let task = FetchService::fetch(request, callback).expect("Failed to start request");
                model.task = Some(task);
            }
            true
        }
    }
}

#[derive(serde::Serialize)]
struct TokenRequest {
    client_id: String,
    client_secret: String,
    audience: String,
    grant_type: String,
}

#[derive(serde::Serialize)]
struct PermissionsForAudience {
    audience: String,
    permissions: Vec<String>,
}

fn get(path: &str) -> Request<Nothing> {
    let request = Request::get(path)
        .header("Content-type", "application/json")
        .body(Nothing)
        .expect("Could not build request");
    request
}

fn post<T: serde::Serialize>(path: &str, body: T) -> Request<Result<String, anyhow::Error>> {
    let request = Request::post(path)
        .header("Content-type", "application/json")
        .body(Ok(serde_json::to_string(&body).unwrap()))
        .expect("Could not build request");
    request
}
