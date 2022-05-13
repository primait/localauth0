use gloo_timers::callback::Timeout;
use std::time::Duration;

use web_sys::HtmlInputElement;
use yew::Context;

#[cfg(not(target_arch = "wasm32"))]
use tokio::task::spawn_local;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

use crate::message::Msg;
use crate::model::{Jwt, Model};

pub fn update(model: &mut Model, context: &Context<Model>, message: Msg) -> bool {
    match message {
        Msg::AudienceFocusOut => {
            if let Some(input) = model.audience_input_ref.cast::<HtmlInputElement>() {
                let audience: String = input.value();
                let url: String = format!("/permissions/{}", &audience);
                model.audience = Some(audience.clone());
                let link = context.link().clone();

                spawn_local(async move {
                    let permissions_opt: Option<Vec<String>> = reqwasm::http::Request::get(url.as_str())
                        .header("Content-type", "application/json")
                        .send()
                        .await
                        .unwrap()
                        .json()
                        .await
                        .unwrap();

                    link.send_message(Msg::ShowPermissions(
                        permissions_opt.unwrap_or_default().into_iter().collect(),
                    ))
                });
            }
            true
        }
        Msg::ShowPermissions(permissions) => {
            model.permissions = permissions;
            true
        }
        Msg::GenerateToken => {
            let audience: String = model.audience.clone().unwrap();
            let link = context.link().clone();

            spawn_local(async move {
                let body: String = serde_json::to_string(&TokenRequest::new(audience)).unwrap();

                let jwt: Jwt = reqwasm::http::Request::post("/oauth/token")
                    .header("Content-type", "application/json")
                    .body(body)
                    .send()
                    .await
                    .unwrap()
                    .json()
                    .await
                    .unwrap();

                link.send_message(Msg::TokenReceived(Some(jwt)))
            });

            false
        }
        Msg::CopyToken => {
            model.do_copy(context);
            false
        }
        Msg::TokenCopied => {
            model.copied = true;
            let link = context.link().clone();
            let timeout_task = Timeout::new(2000, move || link.send_message(Msg::ResetCopyButton));
            model.timeout_task = Some(timeout_task);
            true
        }
        Msg::CopyFailed => {
            model.copied = false;
            true
        }
        Msg::ResetCopyButton => {
            model.copied = false;
            true
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
                let request = PermissionsForAudience {
                    audience: model.audience.clone().unwrap(),
                    permissions: model.permissions.clone().into_iter().collect(),
                };

                let link = context.link().clone();

                spawn_local(async move {
                    let body: String = serde_json::to_string(&request).unwrap();
                    let response: String = reqwasm::http::Request::post("/permissions")
                        .header("Content-type", "application/json")
                        .body(body)
                        .send()
                        .await
                        .unwrap()
                        .text()
                        .await
                        .unwrap();

                    log::info!("Actual mapping for given audience is: {}", response);

                    link.send_message(Msg::GenerateToken)
                });
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

impl TokenRequest {
    fn new(audience: String) -> Self {
        Self {
            audience,
            client_id: "client_id".to_string(),
            client_secret: "client_secret".to_string(),
            grant_type: "client_credentials".to_string(),
        }
    }
}

#[derive(serde::Serialize)]
struct PermissionsForAudience {
    audience: String,
    permissions: Vec<String>,
}
