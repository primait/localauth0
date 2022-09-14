use std::str::FromStr;

use serde::Deserialize;
use url::Url;
use yew::{html, Component, Context, Html};
use yew_router::prelude::{Location, RouterScopeExt};

use msg::Msg;

use crate::pages::model::Jwt;
use crate::pages::{bindgen, bridge};

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
            Msg::CodeRecieved(code) => {
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
                            let () = bridge::login(ctx, |code| Msg::CodeRecieved(code), query_params.audience.clone());
                            html! { <div>{"Loading.."}</div>}
                        }
                        None if self.login_pressed => {
                            let () = bridge::login(ctx, |code| Msg::CodeRecieved(code), query_params.audience.clone());
                            html! { <div>{"Loading.."}</div>}
                        }
                        None => code_login_view(ctx),
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
                            token_login_view(url)
                        }
                    },
                }
            }
        }
    }
}

fn token_login_view(redirect_uri: Url) -> Html {
    html! {
        <div class="form-grid">
            <div class="form-grid__row form-grid__row--small">
                <div class="form-grid__row__column">
                    <div class="button-row button-row--center">
                        <a class="button button--primary button--huge" href={redirect_uri.to_string()}>{"Login"}</a>
                    </div>
                </div>
            </div>
        </div>
    }
}

fn code_login_view(ctx: &Context<SSO>) -> Html {
    html! {
        <div class="form-grid">
            <div class="form-grid__row form-grid__row--small">
                <div class="form-grid__row__column">
                    <div class="button-row button-row--center">
                        <a class="button button--primary button--huge" type="button" onclick={ctx.link().callback(|_|Msg::LoginPressed)}>{"Login"}</a>
                    </div>
                </div>
            </div>
        </div>
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
