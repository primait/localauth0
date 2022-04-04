use yew::format::Text;
use yew::prelude::*;
use yew::services::fetch::{FetchTask, Request, Response};
use yew::services::FetchService;
use yew::web_sys::HtmlInputElement;

use crate::message::Msg;

pub(crate) struct Model {
    link: ComponentLink<Self>,
    audience_input_ref: NodeRef,
    permission_input_ref: NodeRef,
    task: Option<FetchTask>,
    audience: String,
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
            audience: "".to_string(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::GenerateToken => {
                if let Some(input) = self.audience_input_ref.cast::<HtmlInputElement>() {
                    self.audience = input.value();
                    let request = Request::post("/oauth/token")
                        .header("Content-type", "application/json")
                        .body(Ok(serde_json::to_string(&TokenRequest {
                            client_id: "client_id".to_string(),
                            client_secret: "client_secret".to_string(),
                            audience: input.value(),
                            grant_type: "client_credentials".to_string(),
                        })
                        .unwrap()))
                        .expect("Could not build request");

                    let callback = self
                        .link
                        .callback(|response: Response<Text>| Msg::TokenReceived(response.into_body().ok()));

                    let task = FetchService::fetch(request, callback).expect("Failed to start request");
                    self.task = Some(task);
                }
                true
            }
            Msg::TokenReceived(opt) => {
                self.audience = opt.unwrap_or_else(|| "cacca".to_string());
                true
            }
            Msg::AddPermission => {
                false
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
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <label for="audience">{ "Audience" }</label>
                <input name="audience" type="text" ref={self.audience_input_ref.clone()}/>
                <label for="permission">{ "Permission" }</label>
                <input name="permission" type="text" ref={self.permission_input_ref.clone()}/>
                <br/>
                <button onclick=self.link.callback(|_| Msg::GenerateToken)>{"Generate token"}</button>
                <button onclick=self.link.callback(|_| Msg::AddPermission)>{"Add permission"}</button>
                <br/>
                <p>{self.audience.as_str()}</p>
            </div>
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
