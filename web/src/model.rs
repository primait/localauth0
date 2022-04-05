use yew::format::Text;
use yew::prelude::*;
use yew::services::fetch::{FetchTask, Request, Response};
use yew::services::FetchService;
use yew::web_sys::HtmlInputElement;

use crate::message::Msg;

pub(crate) struct Model {
    audience_input_ref: NodeRef,
    permission_input_ref: NodeRef,
    audience: Option<String>,
    permissions: Vec<String>,
    token: Option<String>,
    link: ComponentLink<Self>,
    task: Option<FetchTask>,
}

impl Component for Model {
    type Message = Msg;

    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            audience_input_ref: NodeRef::default(),
            permission_input_ref: NodeRef::default(),
            task: None,
            audience: None,
            permissions: vec![],
            token: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::UpdateAudience => {
                if let Some(input) = self.audience_input_ref.cast::<HtmlInputElement>() {
                    self.audience = Some(input.value());
                }
                true
            }
            Msg::GenerateToken => {
                let request = Request::post("/oauth/token")
                    .header("Content-type", "application/json")
                    .body(Ok(serde_json::to_string(&TokenRequest {
                        client_id: "client_id".to_string(),
                        client_secret: "client_secret".to_string(),
                        audience: self.audience.clone().unwrap(),
                        grant_type: "client_credentials".to_string(),
                    })
                    .unwrap()))
                    .expect("Could not build request");

                let callback = self
                    .link
                    .callback(|response: Response<Text>| Msg::TokenReceived(response.into_body().ok()));

                let task = FetchService::fetch(request, callback).expect("Failed to start request");
                self.task = Some(task);
                false
            }
            Msg::TokenReceived(token_opt) => {
                self.token = token_opt;
                true
            }
            Msg::AddPermission => {
                if let Some(input) = self.permission_input_ref.cast::<HtmlInputElement>() {
                    self.permissions.push(input.value());
                }

                true
            }
            Msg::RemovePermission(permission) => {
                self.permissions = self.permissions.clone().into_iter().filter(|v| v != &permission).collect();
                true
            }

            Msg::SetPermissions => {
                let request = Request::post("/permissions")
                    .header("Content-type", "application/json")
                    .body(Ok(serde_json::to_string(&PermissionsForAudience {
                        audience: self.audience.clone().unwrap(),
                        permissions: self.permissions.clone(),
                    })
                    .unwrap()))
                    .expect("Could not build request");

                log::info!("{} - {:?}", self.audience.clone().unwrap(), self.permissions.as_slice());

                let callback = self
                    .link
                    .callback(|response: Response<Text>| Msg::TokenReceived(response.into_body().ok()));

                let task = FetchService::fetch(request, callback).expect("Failed to start request");
                self.task = Some(task);
                true
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="container spacing-v-xl">
                <h1 class="title-xl-bold">{ "Localauth0" }</h1>
                <div class="form-grid">
                    <div class="form-grid__row">
                        <div class="form-grid__row__column">
                            <div class="form-item">
                                <label class="form-label" for="audience">{ "Audience" }</label>
                                <div class="form-item__wrapper">
                                    <div class="form-field">
                                        <label class="form-field__wrapper">
                                            <input id="form-item-name" class="form-field__text" name="audience" type="text" ref={self.audience_input_ref.clone()}
                                                onkeypress=self.link.batch_callback(|e: KeyboardEvent| {
                                                    if e.key() == "Enter" { Some(Msg::UpdateAudience) } else { None }
                                                })
                                            />
                                        </label>
                                    </div>
                                </div>
                            </div>
                        </div>
                        <div class="form-grid__row__column">
                            <div class="form-item">
                                <label class="form-label" for="permission">{ "Permission" }</label>
                                {{self.permission_input_view()}}
                            </div>
                        </div>
                        <div class="form-grid__row__column"></div>
                        <div class="form-grid__row__column"></div>
                    </div>
                    <div class="">

                    </div>
                    <div class="form-grid__row">
                        <div class="form-grid__row__column">
                            <div class="form-item" style="width: 80px;">
                                <button class="button button--brand button--huge" disabled={self.audience.is_empty()} onclick=self.link.callback(|_| Msg::GenerateToken)>{"Generate token"}</button>
                            </div>
                        </div>
                        <div class="form-grid__row__column">
                            <div class="form-item" style="width: 80px;">
                                <button class="button button--brand button--huge" disabled={self.permissions.is_empty()} onclick=self.link.callback(|_| Msg::SetPermissions)>{"Set permissions"}</button>
                            </div>
                        </div>
                        <div class="form-grid__row__column"></div>
                        <div class="form-grid__row__column"></div>
                    </div>
                </div>

                <p>{self.audience.clone().unwrap_or("-".to_string())}</p>
                { for self.permissions.iter().map(|e| self.view_entry(e.to_string())) }

                <p>{self.token.clone().unwrap_or("-".to_string())}</p>
            </div>
        }
    }
}

impl Model {
    fn permission_input_view(&self) -> Html {
        html! {
            <div class="form-item__wrapper">
                <div class="form-field">
                    <label class="form-field__wrapper">
                        <input id="form-item-name" class="form-field__text" type="text" ref=self.permission_input_ref.clone()
                            onkeypress=self.link.batch_callback(|e: KeyboardEvent| {
                                if e.key() == "Enter" { Some(Msg::AddPermission) } else { None }
                            })
                        />
                    </label>
                </div>
            </div>
        }
    }

    fn view_entry(&self, permission: String) -> Html {
        html! {
            <li>
            {permission.clone()}
            <button class="button button--primary button--medium button--icon-only" onclick=self.link.callback(move |_| Msg::RemovePermission(permission.clone()))>{"-"}</button>
            </li>
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
