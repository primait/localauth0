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
                <div class="container pt-6">
                    <div class="columns is-centered">
                        <div class="column is-half">
                            <div class="level">
                                <div class="level-item has-text-centered">
                                    <figure class="image logo">
                                        <img src="assets/static/media/localauth0.png" width="80" height="80" alt="Localauth0 logo" />
                                    </figure>
                                </div>
                            </div>
                        </div>
                    </div>
                    <div class="columns is-centered">
                        <div class="column is-half">
                            <div class="level">
                                <div class="level-item has-text-centered">
                                    <h4 class="title is-4">{"LOCALAUTH0"}</h4>
                                </div>
                            </div>
                        </div>
                    </div>

                    <Switch<Route> render={Switch::render(route::switch)} />
                </div>
            </BrowserRouter>
        }
    }
}
