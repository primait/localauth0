use std::str::FromStr;

use msg::Msg;
use serde::Deserialize;
use url::Url;
use yew::{Component, Context, html, Html};
use yew_router::prelude::{Location, RouterScopeExt};

use crate::pages::{bindgen, bridge};
use crate::pages::model::Jwt;

mod msg;

const MISSING_PARAMS_CONTENT: &str = "Bad request while authenticating with sso:\
Missing some query params.\
Mandatory query params are: `client_id`, `connection`, audience`, `redirect_uri`, `scope` and `response_type`\
Optional query params are: `state` and `bypass`";

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct QueryParams {
    client_id: String,
    connection: String,
    audience: String,
    redirect_uri: String,
    scope: String,
    response_type: String,
    state: Option<String>,
    bypass: Option<bool>,
}

enum View<'a> {
    Token(Url),
    Code(&'a Context<SSO>),
}

pub struct SSO {
    query_params_opt: Option<QueryParams>,
    token: Option<Jwt>,
    code: Option<String>,
    login_pressed: bool,
}

impl Component for SSO {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let query_params_opt = ctx
            .link()
            .location()
            .and_then(|location| location.query::<QueryParams>().ok());

        Self {
            query_params_opt,
            token: None,
            code: None,
            login_pressed: false,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::TokenReceived(jwt) => {
                self.token = Some(jwt);
                true
            }
            Msg::CodeReceived(code) => {
                self.code = Some(code);
                true
            }
            Msg::LoginPressed => {
                self.login_pressed = true;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match &self.query_params_opt {
            None => error_page(MISSING_PARAMS_CONTENT),
            Some(query_params) => {
                let response_type: String = query_params.response_type.clone();

                match Url::from_str(query_params.redirect_uri.as_str()) {
                    Err(_) => {
                        let error: String = format!(
                            "Provided redirect uri is not valid: {}",
                            query_params.redirect_uri.as_str()
                        );
                        error_page(error.as_str())
                    }
                    Ok(_) if response_type != "token" && response_type != "code" => {
                        let error: String = format!("Provided response type is not valid: {}", response_type);
                        error_page(error.as_str())
                    }
                    Ok(url) if response_type == "code" => match self.code.clone() {
                        Some(code) => {
                            let url: Url = build_code_url(query_params.state.as_ref(), url, code);
                            let _ = bindgen::redirect(url);
                            html! { <div></div> }
                        }
                        None if Some(true) == query_params.bypass => {
                            let () = bridge::login(ctx, |code| Msg::CodeReceived(code), query_params.audience.clone());
                            html! { <div>{"Loading.."}</div>}
                        }
                        None if self.login_pressed => {
                            let () = bridge::login(ctx, |code| Msg::CodeReceived(code), query_params.audience.clone());
                            html! { <div>{"Loading.."}</div>}
                        }
                        None => login_view(View::Code(ctx)),
                    },
                    Ok(url) => match self.token.clone() {
                        None => {
                            let () =
                                bridge::generate_token(ctx, |v| Msg::TokenReceived(v), query_params.audience.clone());
                            html! { <div>{"Loading.."}</div> }
                        }
                        Some(token) if Some(true) == query_params.bypass => {
                            let url: Url = build_token_url(query_params.state.as_ref(), url, token);
                            let _ = bindgen::redirect(url);
                            html! { <div></div> }
                        }
                        Some(token) => {
                            let url: Url = build_token_url(query_params.state.as_ref(), url, token);
                            login_view(View::Token(url))
                        }
                    },
                }
            }
        }
    }
}

// Redirects browser directly to redirect uri
fn login_view(view: View) -> Html {
    html! {
        <div class="columns is-centered">
            <div class="column is-half">
                <div class="level">
                    <div class="level-item has-text-centered pt-6">
                        {{button_for_view(view)}}
                    </div>
                </div>
            </div>
        </div>
    }
}

fn button_for_view(view: View) -> Html {
    match view {
        View::Token(url) => html! {
            <a class="button is-large is-responsive is-success is-light is-outlined" type="button" href={url.to_string()}>{"Login"}</a>
        },
        View::Code(ctx) => html! {
            // When users are supported this view can collect credentials to forward to the backend, but currently no credentials are required.
            <a class="button is-large is-responsive is-success is-light is-outlined" type="button" onclick={ctx.link().callback(|_|Msg::LoginPressed)}>{"Login"}</a>
        }
    }
}

fn error_page(message: &str) -> Html {
    html! {
        <span class="title-xl-bold">{message}</span>
    }
}

fn build_token_url(state_opt: Option<&String>, mut url: Url, token: Jwt) -> Url {
    let state: String = state_opt.map(|state| format!("&state={}", state)).unwrap_or_default();

    let access_token: String = format!(
        "access_token={}&token_type=Bearer&expires_in=3600{}",
        token.access_token(),
        state
    );

    url.set_fragment(Some(access_token.as_str()));
    url
}

fn build_code_url(state_opt: Option<&String>, mut url: Url, code: String) -> Url {
    let state: String = state_opt.map(|state| format!("&state={}", state)).unwrap_or_default();

    let code: String = format!("code={}{}", code, state);

    url.set_query(Some(code.as_str()));
    url
}
