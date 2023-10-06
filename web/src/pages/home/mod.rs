use std::collections::HashSet;

use crate::pages::{bindgen, bridge};
use gloo_timers::callback::Timeout;
use web_sys::HtmlInputElement;
use yew::prelude::{html, Component, Html, NodeRef};
use yew::Context;

use crate::pages::home::msg::Msg;
use crate::pages::model::Jwt;
use crate::util::IsEmpty;

mod msg;

const ENTER_KEY: u32 = 13;

pub struct Home {
    pub audience_input_ref: NodeRef,
    pub permission_input_ref: NodeRef,
    pub audience: Option<String>,
    pub permissions: HashSet<String>,
    pub token: Option<Jwt>,
    pub timeout_task: Option<Timeout>,
    pub copied: bool,
}

impl Component for Home {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            audience_input_ref: NodeRef::default(),
            permission_input_ref: NodeRef::default(),
            timeout_task: None,
            audience: None,
            permissions: HashSet::new(),
            token: None,
            copied: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::AudienceFocusOut => {
                if let Some(input) = self.audience_input_ref.cast::<HtmlInputElement>() {
                    let audience: String = input.value();
                    self.audience = Some(audience.clone());
                    bridge::get_permissions_by_audience(ctx, |v| Msg::ShowPermissions(v), audience)
                }
                true
            }
            Msg::ShowPermissions(permissions) => {
                self.permissions = permissions;
                true
            }
            Msg::GenerateToken => {
                let audience: String = self.audience.clone().unwrap();
                bridge::generate_token(ctx, |v| Msg::TokenReceived(v), audience);
                false
            }
            Msg::CopyToken => {
                match &self.token {
                    None => (),
                    Some(token) => bindgen::copy_to_clipboard(
                        ctx,
                        token.access_token().to_string(),
                        |_| Msg::TokenCopied,
                        |_| Msg::CopyFailed,
                    ),
                }
                false
            }
            Msg::TokenCopied => {
                self.copied = true;
                let link = ctx.link().clone();
                let timeout_task = Timeout::new(2000, move || link.send_message(Msg::ResetCopyButton));
                self.timeout_task = Some(timeout_task);
                true
            }
            Msg::CopyFailed => {
                self.copied = false;
                true
            }
            Msg::ResetCopyButton => {
                self.copied = false;
                true
            }
            Msg::TokenReceived(token) => {
                self.token = Some(token);
                true
            }
            Msg::AddPermission => {
                if let Some(input) = self.permission_input_ref.cast::<HtmlInputElement>() {
                    let value: String = input.value();
                    if !value.is_empty() {
                        self.permissions.insert(input.value());
                        input.set_value("");
                    }
                }
                true
            }
            Msg::RemovePermission(permission) => {
                self.permissions = self
                    .permissions
                    .clone()
                    .into_iter()
                    .filter(|v| v != &permission)
                    .collect();
                true
            }
            Msg::SetPermissions => {
                if let Some(input) = self.audience_input_ref.cast::<HtmlInputElement>() {
                    let audience: String = input.value();
                    self.audience = Some(audience.clone());
                    bridge::set_permissions_for_audience(ctx, || Msg::GenerateToken, audience, self.permissions.clone())
                }

                true
            }
            Msg::PermissionKeyUp(event) => {
                if ENTER_KEY == event.key_code() {
                    self.update(ctx, Msg::AddPermission)
                } else {
                    false
                }
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onfocusout = ctx.link().callback(|_| Msg::AudienceFocusOut);

        html! {
            <div>
                // Audience form
                <div class="columns is-centered pt-5">
                    <div class="column is-one-third">
                        <div class="field">
                            <label class="label">{"Audience"}</label>
                            <div class="control">
                                <input class="input" type="text" placeholder="Audience" onblur={onfocusout} ref={self.audience_input_ref.clone()}/>
                            </div>
                        </div>
                    </div>
                </div>
                // Permissions form
                {
                    if !self.audience.is_empty() {
                        html! {
                            <div class="columns is-centered">
                                <div class="column is-one-third">
                                    <div class="field">
                                        <label class="label">{"Permissions"}</label>
                                        <div class="columns">
                                            <div class="column is-four-fifths">
                                                <input class="input" type="text" placeholder="Permissions" ref={&self.permission_input_ref.clone()} onkeyup={ctx.link().callback(|e| Msg::PermissionKeyUp(e))}/>
                                            </div>
                                            <div class="column level">
                                                <div class="level-right">
                                                    <a class="button is-responsive is-success is-light is-outlined" type="button" onclick={ctx.link().batch_callback(|_| { Some(Msg::AddPermission) })}>
                                                        <div aria-hidden="false" aria-label="Add permission" class="icon icon--size-l" role="img">
                                                            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
                                                                <path d="M22.281 11.5H12.5V1.719a.5.5 0 1 0-1 0V11.5H1.719a.5.5 0 1 0 0 1H11.5v9.781a.5.5 0 0 0 1 0V12.5h9.781a.5.5 0 0 0 0-1z"></path>
                                                            </svg>
                                                        </div>
                                                    </a>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }
                // Permission entries
                {
                    if !self.permissions.is_empty() {
                        html! {
                            <div class="pt-4 pb-6">
                            {
                                for self.permissions.iter().map(|permission| permission_entry(ctx, permission.to_string()))
                            }
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }
                // Token form
                <div class="columns is-centered">
                    <div class="column is-one-third">
                        <div class="field">
                            <label class="label">{"Token"}</label>
                            <div class="control">
                                <textarea class="textarea" value={self.token.as_ref().map(|jwt| jwt.access_token().to_string()).unwrap_or_else(|| "No token".to_string())} readonly=true></textarea>
                                <div class="copy-wrapper">
                                    <span class="tag button-copy" onclick={ctx.link().callback(|_| Msg::CopyToken)}>{if self.copied { "Copied!" } else { "Copy" } }</span>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
                // Generate token button
                <div class="columns is-centered">
                    <div class="column is-one-third">
                        <div class="level">
                            <div class="level-item has-text-centered">
                                <button class="button is-responsive is-success is-light is-outlined" type="button" disabled={self.audience.is_empty()} onclick={ctx.link().callback(|_| Msg::SetPermissions)}>{"Generate token"}</button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}

fn permission_entry(ctx: &Context<Home>, permission: String) -> Html {
    html! {
        <div class="columns is-centered padding-03">
            <div class="column is-one-third padding-03">
                <div class="field">
                    <div class="columns">
                        <div class="column is-four-fifths padding-03">
                            <p style="padding-top: 5px;">{permission.clone()}</p>
                        </div>
                        <div class="column level padding-03">
                            <div class="level-right">
                                <a class="button is-small is-responsive is-success is-light is-outlined" type="button" onclick={ctx.link().callback(move |_| Msg::RemovePermission(permission.clone()))}>
                                    <div aria-hidden="false" aria-label="Add permission" class="icon icon--size-l" role="img">
                                        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
                                            <path d="M12.714 11.976l9.121-9.121a.5.5 0 1 0-.707-.707l-9.121 9.121-9.121-9.121a.5.5 0 0 0-.707.707l9.121 9.121-9.121 9.121a.5.5 0 1 0 .707.707l9.121-9.121 9.121 9.121a.5.5 0 1 0 .707-.707z"></path>
                                        </svg>
                                    </div>
                                </a>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
