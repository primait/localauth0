use url::Url;
use wasm_bindgen::prelude::*;
use yew::{Callback, Component, Context};

#[cfg(not(target_arch = "wasm32"))]
use tokio::task::spawn_local;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

pub fn copy_to_clipboard<T, S, E>(ctx: &Context<T>, value: String, success_fn: S, error_fn: E)
where
    T: Component,
    S: 'static + Fn(((),)) -> T::Message,
    E: 'static + Fn(((),)) -> T::Message,
{
    let ok: Callback<((),)> = ctx.link().callback(success_fn);
    let err: Callback<((),)> = ctx.link().callback(error_fn);

    spawn_local(async move {
        match js_copy_to_clipboard(value).await {
            Ok(_) => ok.emit(((),)),
            Err(_) => err.emit(((),)),
        };
    });
}

pub fn redirect(url: Url) {
    spawn_local(async move {
        let _ = js_redirect(url.to_string()).await;
    });
}

#[wasm_bindgen(inline_js=r#"
export function js_copy_to_clipboard(value) {
    try {
        return window.navigator.clipboard.writeText(value);
    } catch(e) {
        console.log(e);
        return Promise.reject(e)
    }
}
"#)]
#[rustfmt::skip] // required to keep the "async" keyword
extern "C" {
    #[wasm_bindgen(catch)]
    async fn js_copy_to_clipboard(value: String) -> Result<(), JsValue>;
}

#[wasm_bindgen(inline_js=r#"
export function js_redirect(value) {
    try {
        return window.location.replace(value);
    } catch(e) {
        console.log(e);
        return Promise.reject(e)
    }
}
"#)]
#[rustfmt::skip] // required to keep the "async" keyword
extern "C" {
    #[wasm_bindgen(catch)]
    async fn js_redirect(value: String) -> Result<(), JsValue>;
}
