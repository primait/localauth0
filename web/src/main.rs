use yew::prelude::{html, Component, Html};
use yew::Context;
use yew_router::prelude::*;

use localauth0_web::route::{self, Route};

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<Model>();
}

pub struct Model {}

impl Component for Model {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <BrowserRouter>
                <div class="container padding-v-l">
                    <div class="form-grid">
                        <div class="form-grid__row form-grid__row--small">
                            <div class="form-grid__row__column">
                                <legend class="form-legend">
                                    <span class="form-legend__addon">
                                        <img src="assets/static/media/localauth0.png" width="80" height="80" alt="Localauth0 logo" />
                                    </span>
                                    <span class="form-legend__title">{"LOCALAUTH0"}</span>
                                </legend>
                            </div>
                        </div>

                        <Switch<Route> render={Switch::render(route::switch)} />

                    </div>
                </div>
            </BrowserRouter>
        }
    }
}
