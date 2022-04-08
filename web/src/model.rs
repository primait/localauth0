use std::collections::HashSet;

use serde::Deserialize;
use wasm_bindgen::prelude::*;
use yew::prelude::{html, Component, ComponentLink, Html, NodeRef, ShouldRender};
use yew::services::fetch::FetchTask;
use yew::services::timeout::TimeoutTask;

use crate::message::Msg;
use crate::update;

pub struct Model {
    pub audience_input_ref: NodeRef,
    pub permission_input_ref: NodeRef,
    pub audience: Option<String>,
    pub permissions: HashSet<String>,
    pub token: Option<Jwt>,
    pub link: ComponentLink<Self>,
    pub fetch_task: Option<FetchTask>,
    pub timeout_task: Option<TimeoutTask>,
    pub copied: bool,
}

impl Component for Model {
    type Message = Msg;

    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            audience_input_ref: NodeRef::default(),
            permission_input_ref: NodeRef::default(),
            fetch_task: None,
            timeout_task: None,
            audience: None,
            permissions: HashSet::new(),
            token: None,
            copied: false,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        update::update(self, msg)
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let onfocusout = self.link.callback(|_| Msg::AudienceFocusOut);

        html! {
            <div class="container padding-v-l">
                <div class="form-grid">
                    <div class="form-grid__row form-grid__row--small">
                        <div class="form-grid__row__column">
                            <legend class="form-legend">
                                <span class="form-legend__addon">
                                    <img
                                        src="assets/static/media/localauth0.png"
                                        width="80"
                                        height="80"
                                        alt="Localauth0 logo"
                                    />
                                </span>
                                <span class="form-legend__title">{"LOCALAUTH0"}</span>
                            </legend>
                        </div>
                    </div>

                    <div class="form-grid__row form-grid__row--small">
                        <div class="form-item">
                            <label class="form-label" for="audience">{ "Audience" }</label>
                            <div class="form-item__wrapper">
                                <div class="form-field">
                                    <label class="form-field__wrapper">
                                        <input id="form-item-name" class="form-field__text" name="audience" type="text" onblur={onfocusout} placeholder="audience" ref={self.audience_input_ref.clone()}/>
                                    </label>
                                </div>
                            </div>
                        </div>
                    </div>

                    {{self.permission_input_view()}}

                    { for self.permissions.iter().map(|permission| self.view_entry(permission.to_string())) }

                    <div class="form-grid__row form-grid__row--small">
                        <div class="form-item">
                            <label class="form-label" for="label-and-textarea">{ "Token" }</label>
                            <div class="form-item__wrapper">
                                <div class="form-field">
                                    <div class="token-area">{self.token.clone().map(|jwt| jwt.access_token).unwrap_or("No token".to_string())}</div>
                                    // <textarea
                                    //     id="label-and-textarea"
                                    //     class="form-field__textarea token-area"
                                    //     readonly=true>
                                    //     {self.token.clone().map(|jwt| jwt.access_token).unwrap_or("No token".to_string())}
                                    // </textarea>
                                    <div class="copy-wrapper">
                                        <span class="badge button-copy" onclick=self.link.callback(|_| Msg::CopyToken)>{if self.copied { "Copied!" } else { "Copy" } }</span>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>

                    <div class="form-grid__row form-grid__row--small">
                        <div class="form-grid__row__column">
                            <div class="button-row button-row--center">
                                <button class="button button--primary button--huge" disabled={self.audience.is_empty()} onclick=self.link.callback(|_| Msg::SetPermissions)>{"Generate token"}</button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}

impl Model {
    fn permission_input_view(&self) -> Html {
        html! {
            <div class="form-grid__row form-grid__row--small">
                <div class="form-grid__row__column form-grid__row__column--span-5">
                    <div class="form-item">
                        <label class="form-label" for="permission">{ "Permission" }</label>
                        <div class="form-item__wrapper">
                            <div class="form-field">
                                <label class="form-field__wrapper">
                                    <input id="form-item-name" class="form-field__text" type="text" placeholder="permission" ref=self.permission_input_ref.clone()/>
                                </label>
                            </div>
                        </div>
                    </div>
                </div>
                <div class="form-grid__row__column display-grid">
                    <button class="button button--primary button--huge button--icon-only permission-button" type="button" onclick=self.link.batch_callback(|_| { Some(Msg::AddPermission) })>
                        <div aria-hidden="false" aria-label="Add permission" class="icon icon--size-l" role="img">
                            {{self.permission_add_icon()}}
                        </div>
                    </button>
                </div>
            </div>
        }
    }

    fn view_entry(&self, permission: String) -> Html {
        html! {
            <div class="form-grid__row form-grid__row--small">
                <div class="form-grid__row__column form-grid__row__column--span-5">
                    <div class="form-item">
                        <div class="form-item__wrapper">
                            <div class="form-field">
                                <label class="form-field__wrapper">
                                    <input id="form-item-name" class="form-field__text" readonly=true type="text" value={permission.clone()} />
                                </label>
                            </div>
                        </div>
                    </div>
                </div>

                <div class="form-grid__row__column display-grid">
                    <button
                        type="button"
                        class="button button--primary button--huge button--icon-only permission-button"
                        onclick=self.link.callback(move |_| Msg::RemovePermission(permission.clone()))>
                        <div aria-hidden="false" aria-label="Remove permission" class="icon icon--size-l" role="img">
                            {{self.permission_delete_icon()}}
                        </div>
                    </button>
                </div>
            </div>
        }
    }

    fn permission_add_icon(&self) -> Html {
        html! {
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24"><path d="M22.281 11.5H12.5V1.719a.5.5 0 1 0-1 0V11.5H1.719a.5.5 0 1 0 0 1H11.5v9.781a.5.5 0 0 0 1 0V12.5h9.781a.5.5 0 0 0 0-1z"></path></svg>
        }
    }

    fn permission_delete_icon(&self) -> Html {
        html! {
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24"><path d="M12.714 11.976l9.121-9.121a.5.5 0 1 0-.707-.707l-9.121 9.121-9.121-9.121a.5.5 0 0 0-.707.707l9.121 9.121-9.121 9.121a.5.5 0 1 0 .707.707l9.121-9.121 9.121 9.121a.5.5 0 1 0 .707-.707z"></path></svg>
        }
    }

    pub fn do_copy(&self) {
        match &self.token {
            None => (),
            Some(jwt) => {
                let ok = self.link.callback(|_| Msg::TokenCopied);
                let err = self.link.callback(|_| Msg::CopyFailed);
                let access_token: String = jwt.access_token().to_string();
                wasm_bindgen_futures::spawn_local(async move {
                    match copy_to_clipboard(access_token).await {
                        Ok(_) => ok.emit(()),
                        Err(_) => err.emit(()),
                    };
                });
            }
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Jwt {
    access_token: String,
}

impl Jwt {
    fn access_token(&self) -> &str {
        &self.access_token
    }
}

trait IsEmpty {
    fn is_empty(&self) -> bool;
}

impl IsEmpty for Option<String> {
    fn is_empty(&self) -> bool {
        match &self {
            None => true,
            Some(string) => string == "",
        }
    }
}

#[wasm_bindgen(inline_js=r#"
export function copy_to_clipboard(value) {
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
    async fn copy_to_clipboard(value: String) -> Result<(), JsValue>;
}
