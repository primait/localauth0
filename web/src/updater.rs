use yew::{
    format::Text,
    services::{
        fetch::{Request, Response},
        FetchService,
    },
    web_sys::HtmlInputElement,
};

use crate::message::Msg;
use crate::model::Model;

pub fn update(model: &mut Model, message: Msg) -> bool {
    match message {
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
                let c = response.into_body().unwrap();
                let body = serde_json::from_str(&c).unwrap();
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
                model.permissions.push(input.value());
                input.set_value("");
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
                        permissions: model.permissions.clone(),
                    },
                );

                let callback = model.link.callback(|response: Response<Text>| Msg::GenerateToken);
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

fn post<T: serde::Serialize>(path: &str, body: T) -> Request<Result<String, anyhow::Error>> {
    let request = Request::post(path)
        .header("Content-type", "application/json")
        .body(Ok(serde_json::to_string(&body).unwrap()))
        .expect("Could not build request");
    request
}
