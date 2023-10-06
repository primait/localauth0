use crate::pages::{Home, SSO};
use yew::prelude::{html, Html};
use yew_router::prelude::Routable;

#[derive(Routable, PartialEq, Clone, Debug)]
pub enum Route {
    #[at("/authorize")]
    SSO,
    #[at("/")]
    Home,
    #[not_found]
    #[at("/404")]
    NotFound,
}

pub fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <Home /> },
        Route::SSO => html! { <SSO /> },
        Route::NotFound => html! { <span class="title-xl-bold">{"Page not found"}</span> },
    }
}
