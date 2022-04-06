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
                                            <input id="form-item-name"  class="form-field__text" name="audience" type="text" ref={self.audience_input_ref.clone()}/>
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
                        <div class="form-grid__row__column">
                            <div class="form-item" style="width: 80px; margin-top: 25px">
                                <label class="form-label" ></label>
                                <button class="button button--brand button--huge" onclick=self.link.callback(|_| Msg::SetPermissions)>{"Generate token"}</button>
                            </div>
                        </div>
                        <div class="form-grid__row__column"></div>
                    </div>
                    <div class="">

                    </div>
                </div>

                <div class="form-grid">
                    <div class="form-grid__row form-grid__row--medium">
                        <div class="form-grid__row__column form-grid__row__column-medium">
                            <label class="form-label" for="label-and-textarea">{ "Token" }</label>
                            <div class="form-item__wrapper">
                                <div class="form-field">
                                    <textarea
                                    class="form-field__textarea"
                                    readonly=true
                                    id="label-and-textarea">
                                    {self.token.clone().map(|jwt| jwt.access_token).unwrap_or("-".to_string())}</textarea>
                                </div>
                            </div>
                        </div>
                        <div class="form-grid form-grid--gap-small form-grid__row__column form-grid__row__column-medium">
                            { for self.permissions.iter().map(|permission| self.view_entry(permission.to_string())) }
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
            <div class="form-item__wrapper">
                <div class="form-field form-grid__row">
                    <div class="form-grid__row__column form-grid__row__column--span-2">
                        <label class="form-field__wrapper">
                            <input id="form-item-name" class="form-field__text" type="text" ref=self.permission_input_ref.clone()/>
                        </label>
                    </div>
                    <div class="form-grid__row__column ">
                        <button class="button button--primary button--large " onclick=self.link.batch_callback(|_| { Some(Msg::AddPermission) })>{"+"}</button>
                    </div>
                </div>
            </div>
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
