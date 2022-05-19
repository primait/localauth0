use std::str::FromStr;

use serde::Deserialize;
use url::Url;
use yew::{html, Component, Context, Html};
use yew_router::history::History;
use yew_router::prelude::{Location, RouterScopeExt};

use msg::Msg;

use crate::pages::model::Jwt;
use crate::pages::{bindgen, bridge};
use crate::route::Route;

mod msg;

const MISSING_PARAMS_CONTENT: &str = "Bad request while authenticating with sso:\
Missing some query params.\
Mandatory query params are: `audience` and `redirect_uri\
Optional query params are: `response_type` and `state`";

#[derive(Deserialize, Debug)]
struct QueryParams {
    audience: String,
    redirect_uri: String,
    response_type: Option<String>,
    state: Option<String>,
    bypass: Option<bool>,
}

pub struct SSO {
    query_params_opt: Option<QueryParams>,
    token: Option<Jwt>,
}

impl Component for SSO {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let query_params_opt = ctx
            .link()
            .location()
            .and_then(|location| location.query::<QueryParams>().ok());

        let bypass: bool = query_params_opt
            .as_ref()
            .map(|query_params| query_params.bypass.unwrap_or(false))
            .unwrap_or(false);
        if bypass {
            let redirect_uri: Option<Url> = query_params_opt
                .as_ref()
                .map(|query_params| query_params.redirect_uri.as_str())
                .and_then(|uri| Url::parse(uri).ok());

            match redirect_uri {
                None => {
                    ctx.link()
                        .history()
                        .expect("Something went wrong getting history")
                        .push(Route::NotFound);
                }
                Some(url) => bindgen::redirect(url),
            }
        }

        Self {
            query_params_opt,
            token: None,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::TokenReceived(jwt) => {
                self.token = Some(jwt);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match &self.query_params_opt {
            None => error_page(MISSING_PARAMS_CONTENT),
            Some(query_params) => {
                let response_type: String = query_params
                    .response_type
                    .clone()
                    .unwrap_or_else(|| "token".to_string());

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
                    Ok(_) if response_type == "code" => error_page("`code` response type is not supported yet. Sorry"),
                    Ok(mut url) => match self.token.clone() {
                        None => {
                            let () =
                                bridge::generate_token(ctx, |v| Msg::TokenReceived(v), query_params.audience.clone());
                            html! { <div>{"Loading.."}</div> }
                        }
                        Some(token) => {
                            let state: String = query_params
                                .state
                                .as_ref()
                                .map(|state| format!("&state={}", state))
                                .unwrap_or_default();

                            let access_token: String = format!(
                                "access_token={}&token_type=Bearer&expires_in=3600{}",
                                token.access_token(),
                                state
                            );
                            url.set_fragment(Some(access_token.as_str()));
                            login_view(url)
                        }
                    },
                }
            }
        }
    }
}

fn login_view(redirect_uri: Url) -> Html {
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

fn error_page(message: &str) -> Html {
    html! {
        <span class="title-xl-bold">{message}</span>
    }
}
