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
                // if let Some(input) = self.permission_input_ref.cast::<HtmlInputElement>() {
                //     self.audience = input.value();
                //     let request = Request::post("/oauth/token")
                //         .header("Content-type", "application/json")
                //         .body(Ok(serde_json::to_string(&TokenRequest {
                //             client_id: "client_id".to_string(),
                //             client_secret: "client_secret".to_string(),
                //             audience: input.value(),
                //             grant_type: "client_credentials".to_string(),
                //         })
                //             .unwrap()))
                //         .expect("Could not build request");
                //
                //     let callback = self
                //         .link
                //         .callback(|response: Response<Text>| Msg::SetText(response.into_body().ok()));
                //
                //     let task = FetchService::fetch(request, callback).expect("Failed to start request");
                //     self.task = Some(task);
                // }
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
            <div>
                <div class="grid">
                    <div>
                        <label for="audience">{ "Audience" }</label>
                        <input name="audience" type="text" ref={self.audience_input_ref.clone()}
                            onkeypress=self.link.batch_callback(|e: KeyboardEvent| {
                                if e.key() == "Enter" { Some(Msg::UpdateAudience) } else { None }
                            })
                        />
                    </div>
                    <div>
                        <label for="permission">{ "Permission" }</label>
                        {{self.view_input()}}
                    </div>
                </div>

                <p>{self.audience.clone().unwrap_or("-".to_string())}</p>
                { for self.permissions.iter().map(|e| self.view_entry(e.to_string())) }

                <br/>
                <button onclick=self.link.callback(|_| Msg::GenerateToken)>{"Generate token"}</button>
                <button onclick=self.link.callback(|_| Msg::SetPermissions)>{"Set permissions"}</button>
                <br/>
                <p>{self.token.clone().unwrap_or("-".to_string())}</p>
            </div>
        }
    }
}

impl Model {
    fn view_input(&self) -> Html {
        html! {
            // You can use standard Rust comments. One line:
            // <li></li>
            <input ref=self.permission_input_ref.clone()
                onkeypress=self.link.batch_callback(|e: KeyboardEvent| {
                    if e.key() == "Enter" { Some(Msg::AddPermission) } else { None }
                })
            />
            /* Or multiline:
            <ul>
                <li></li>
            </ul>
            */
        }
    }

    fn view_entry(&self, permission: String) -> Html {
        html! {
            <li>
            {permission.clone()}
            <button onclick=self.link.callback(move |_| Msg::RemovePermission(permission.clone()))>{"-"}</button>
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
