use std::collections::HashSet;

use reqwasm::http::Request;
#[cfg(not(target_arch = "wasm32"))]
use tokio::task::spawn_local;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;
use yew::html::Scope;
use yew::{Component, Context};

use crate::pages::model::{Jwt, PermissionsForAudience, TokenRequest, LoginRequest, LoginResponse};

pub fn generate_token<T, F>(ctx: &Context<T>, msg: F, audience: String)
where
    T: Component,
    F: 'static + FnOnce(Jwt) -> T::Message,
{
    let link: Scope<T> = ctx.link().clone();
    spawn_local(async move {
        let body: String = serde_json::to_string(&TokenRequest::new(audience)).unwrap();

        let jwt: Jwt = Request::post("/oauth/token")
            .header("Content-type", "application/json")
            .body(body)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        link.send_message(msg(jwt))
    });
}

pub fn get_permissions_by_audience<T, F>(ctx: &Context<T>, msg: F, audience: String)
where
    T: Component,
    F: 'static + FnOnce(HashSet<String>) -> T::Message,
{
    let url: String = format!("/permissions/{}", &audience);
    let link: Scope<T> = ctx.link().clone();
    spawn_local(async move {
        let permissions_opt: Option<Vec<String>> = Request::get(url.as_str())
            .header("Content-type", "application/json")
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        let permissions: HashSet<String> = permissions_opt.unwrap_or_default().into_iter().collect();

        link.send_message(msg(permissions))
    });
}

pub fn set_permissions_for_audience<T, F>(ctx: &Context<T>, msg: F, audience: String, permissions: HashSet<String>)
where
    T: Component,
    F: 'static + FnOnce() -> T::Message,
{
    let request: PermissionsForAudience = PermissionsForAudience::new(audience, permissions.into_iter().collect());
    let link: Scope<T> = ctx.link().clone();
    spawn_local(async move {
        let body: String = serde_json::to_string(&request).unwrap();
        let response: String = Request::post("/permissions")
            .header("Content-type", "application/json")
            .body(body)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        log::info!("Actual mapping for given audience is: {}", response);

        link.send_message(msg())
    });
}

pub fn login<T, F>(ctx: &Context<T>, msg: F, audience: String)
where
    T: Component,
    F: 'static + FnOnce(String) -> T::Message,
{
    let link: Scope<T> = ctx.link().clone();
    spawn_local(async move {
        let body: String = serde_json::to_string(&LoginRequest::new(audience)).unwrap();

        let response: LoginResponse = Request::post("/oauth/login")
            .header("Content-type", "application/json")
            .body(body)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        link.send_message(msg(response.code))
    });
}