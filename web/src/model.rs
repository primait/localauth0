use serde::Deserialize;
use yew::prelude::{html, Component, ComponentLink, Html, KeyboardEvent, NodeRef, ShouldRender};
use yew::services::fetch::FetchTask;

use crate::message::Msg;
use crate::updater;

pub struct Model {
    pub audience_input_ref: NodeRef,
    pub permission_input_ref: NodeRef,
    pub audience: Option<String>,
    pub permissions: Vec<String>,
    pub token: Option<Jwt>,
    pub link: ComponentLink<Self>,
    pub task: Option<FetchTask>,
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
        updater::update(self, msg)
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="container main spacing-v-xl">
                <div class="form-grid">
                    // Title
                    <div class="form-grid__row form-grid__row--small">
                        <legend class="form-legend">
                            <span class="form-legend__title">{"LOCALAUTH0"}</span>
                            <span class="form-legend__addon">
                                <img
                                    src="assets/static/media/localauth0.png"
                                    width="80"
                                    height="80"
                                    alt="Localauth0 logo"
                                />
                            </span>
                            //<span class="form-legend__text">Legend description</span>
                        </legend>
                    </div>

                    // Audience input
                    <div class="form-grid__row form-grid__row--small">
                        <div class="form-item">
                            <label class="form-label" for="audience">{ "Audience" }</label>
                            <div class="form-item__wrapper">
                                <div class="form-field">
                                    <label class="form-field__wrapper">
                                        <input id="form-item-name" class="form-field__text" name="audience" type="text" placeholder="audience" ref={self.audience_input_ref.clone()}/>
                                    </label>
                                </div>
                            </div>
                        </div>
                    </div>

                    {{self.permission_input_view()}}

                    <div class="form-grid__row form-grid__row--small">
                        { for self.permissions.iter().map(|permission| self.view_entry(permission.to_string())) }
                    </div>

                    <div class="form-grid__row form-grid__row--small">
                        <div class="form-item">
                            <label class="form-label" for="label-and-textarea">{ "Token" }</label>
                            <div class="form-item__wrapper">
                                <div class="form-field">
                                    <textarea
                                        id="label-and-textarea"
                                        class="form-field__textarea token-area"
                                        readonly=true
                                    >
                                        {self.token.clone().map(|jwt| jwt.access_token).unwrap_or("No token".to_string())}
                                    </textarea>
                                </div>
                            </div>
                        </div>
                    </div>

                    <div class="form-grid__row form-grid__row--small">
                        <div class="button-row">
                            <button class="button button--primary button--huge" onclick=self.link.callback(|_| Msg::SetPermissions)>{"Generate token"}</button>
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}

impl Model {
    fn permission_input_view(&self) -> Html {
        html! {
            <div class="form-grid__row form-grid__row--small">
                <div class="form-grid__row__column form-grid__row__column--span-5">
                    <div class="form-item">
                        <label class="form-label" for="permission">{ "Permission" }</label>
                        <div class="form-item__wrapper">
                            <div class="form-field">
                                <label class="form-field__wrapper">
                                    <input id="form-item-name" class="form-field__text" type="text" placeholder="permission" ref=self.permission_input_ref.clone()/>
                                </label>
                            </div>
                        </div>
                    </div>
                </div>
                <div class="form-grid__row__column display-grid">
                    <button class="button button--primary button--huge button--icon-only permission-button" type="button" onclick=self.link.batch_callback(|_| { Some(Msg::AddPermission) })>
                        <div aria-hidden="false" aria-label="Button" class="icon icon--size-s" role="img">
                            {{self.permission_icon()}}
                        </div>
                    </button>
                </div>
            </div>


            // <div class="form-item">
            //     <label class="form-label" for="permission">{ "Permission" }</label>
            //     <div class="form-item__wrapper">
            //         <div class="form-field">
            //             <label class="form-field__wrapper">
            //                 <input id="form-item-name" class="form-field__text" type="text" ref=self.permission_input_ref.clone()/>
            //             </label>
            //         </div>
            //         <div class="form-field">
            //             <button class="button button--primary button--large " onclick=self.link.batch_callback(|_| { Some(Msg::AddPermission) })>{"+"}</button>
            //         </div>
            //     </div>
            // </div>
        }
    }

    fn view_entry(&self, permission: String) -> Html {
        html! {
            <div class="form-grid__row">
                <div class="form-grid__row__column form-grid__row__column--small">{&permission}</div>
                <div class="form-grid__row__column form-grid__row__column--span-3">
                    <button class="button button--primary button--medium button--icon-only" onclick=self.link.callback(move |_| Msg::RemovePermission(permission.clone()))>{"-"}</button>
                </div>
            </div>
        }
    }

    fn permission_icon(&self) -> Html {
        html! {
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24"><path d="M22.281 11.5H12.5V1.719a.5.5 0 1 0-1 0V11.5H1.719a.5.5 0 1 0 0 1H11.5v9.781a.5.5 0 0 0 1 0V12.5h9.781a.5.5 0 0 0 0-1z"></path></svg>
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Jwt {
    access_token: String,
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
