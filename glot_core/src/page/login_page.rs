use crate::common::browser_context::BrowserContext;
use crate::common::quick_action;
use crate::common::quick_action::LanguageQuickAction;
use crate::components::search_modal;
use crate::layout::app_layout;
use glot_languages::language;
use maud::html;
use maud::Markup;
use poly::browser::dom_id::DomId;
use poly::browser::effect;
use poly::browser::effect::Effect;
use poly::browser::subscription;
use poly::browser::subscription::event_listener;
use poly::browser::subscription::Subscription;
use poly::browser::value::Capture;
use poly::page::JsMsg;
use poly::page::Page;
use poly::page::PageMarkup;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Model {
    pub browser_ctx: BrowserContext,
    pub layout_state: app_layout::State,
    pub search_modal_state: search_modal::State<LanguageQuickAction>,
    pub email: String,
    pub form_state: FormState,
}

#[derive(Serialize, Deserialize)]
enum FormState {
    NotSubmitted,
    Submitted,
    Error(String),
}

pub struct LoginPage {
    pub browser_ctx: BrowserContext,
}

impl Page<Model, Msg, Markup> for LoginPage {
    fn id(&self) -> &'static dyn DomId {
        &Id::Glot
    }

    fn init(&self) -> Result<(Model, Effect<Msg>), String> {
        let model = Model {
            layout_state: app_layout::State::default(),
            browser_ctx: self.browser_ctx.clone(),
            search_modal_state: search_modal::State::default(),
            email: "".to_string(),
            form_state: FormState::NotSubmitted,
        };

        Ok((model, effect::none()))
    }

    fn subscriptions(&self, model: &Model) -> Subscription<Msg> {
        subscription::batch(vec![
            event_listener::on_click_closest(Id::QuickActionButton, Msg::QuickActionButtonClicked),
            event_listener::on_click_closest(Id::SendLinkButton, Msg::LoginFormSubmitted),
            event_listener::on_submit(Id::LoginForm, Msg::LoginFormSubmitted),
            event_listener::on_input(Id::Email, Msg::EmailChanged),
            app_layout::subscriptions(&model.layout_state, Msg::AppLayoutMsg),
            search_modal::subscriptions(
                &model.browser_ctx.user_agent,
                &model.search_modal_state,
                Msg::SearchModalMsg,
            ),
        ])
    }

    fn update(&self, msg: &Msg, model: &mut Model) -> Result<Effect<Msg>, String> {
        match msg {
            Msg::QuickActionButtonClicked => {
                // fmt
                Ok(model.search_modal_state.open())
            }

            Msg::SearchModalMsg(child_msg) => {
                let data: search_modal::UpdateData<Msg, LanguageQuickAction> =
                    search_modal::update(
                        child_msg,
                        &mut model.search_modal_state,
                        quick_action::language_entries(),
                        Msg::SearchModalMsg,
                    )?;

                let effect = data
                    .action
                    .map(|entry| entry.perform_action(&model.browser_ctx.current_url))
                    .unwrap_or_else(effect::none);

                Ok(effect::batch(vec![effect, data.effect]))
            }

            Msg::AppLayoutMsg(child_msg) => {
                let event = app_layout::update(child_msg, &mut model.layout_state)?;
                match event {
                    app_layout::Event::None => Ok(effect::none()),
                    app_layout::Event::OpenSearch => Ok(model.search_modal_state.open()),
                }
            }

            Msg::EmailChanged(captured) => {
                model.email = captured.value();
                Ok(effect::none())
            }

            Msg::LoginFormSubmitted => {
                let req = SendLoginLinkRequest {
                    email: model.email.clone(),
                };
                let effect = effect::custom(CustomEffect::SendLoginLink(req));
                Ok(effect)
            }
        }
    }

    fn update_from_js(&self, msg: JsMsg, model: &mut Model) -> Result<Effect<Msg>, String> {
        match msg.type_.as_ref() {
            "GotSendLoginLinkResponse" => {
                let response: Response = serde_json::from_value(msg.data)
                    .map_err(|err| format!("Failed to decode response from js: {}", err))?;

                if response.is_ok() {
                    model.form_state = FormState::Submitted;
                } else {
                    model.form_state = FormState::Error(format!(
                        "Failed to send login link. Status code: {}, body: {}",
                        response.status, response.body
                    ));
                }

                Ok(effect::none())
            }

            _ => Err(format!("Unknown message type: {}", msg.type_)),
        }
    }

    fn view(&self, model: &Model) -> PageMarkup<Markup> {
        PageMarkup {
            head: view_head(model),
            body: view_body(model),
        }
    }

    fn render(&self, markup: Markup) -> String {
        markup.into_string()
    }

    fn render_page(&self, markup: PageMarkup<Markup>) -> String {
        app_layout::render_page(markup)
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub status: u16,
    pub body: String,
}

impl Response {
    pub fn is_ok(&self) -> bool {
        self.status >= 200 && self.status <= 299
    }

    pub fn decode<T: for<'de> Deserialize<'de>>(&self) -> Result<T, String> {
        serde_json::from_str(&self.body)
            .map_err(|err| format!("Failed to decode response body: {}", err))
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "config")]
#[serde(rename_all = "camelCase")]
pub enum CustomEffect {
    SendLoginLink(SendLoginLinkRequest),
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct SendLoginLinkRequest {
    email: String,
}

#[derive(strum_macros::Display, poly_macro::DomId)]
#[strum(serialize_all = "kebab-case")]
enum Id {
    Glot,
    QuickActionButton,
    Email,
    SendLinkButton,
    LoginForm,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Msg {
    AppLayoutMsg(app_layout::Msg),
    // Search modal related
    QuickActionButtonClicked,
    SearchModalMsg(search_modal::Msg),
    // Page specific
    EmailChanged(Capture<String>),
    LoginFormSubmitted,
}

fn view_head(_model: &Model) -> maud::Markup {
    let description = "glot.io login page";

    html! {
        title { "login - glot.io" }
        meta name="description" content=(description);
        meta name="viewport" content="width=device-width, initial-scale=1";
        link rel="stylesheet" href="/static/app.css?hash=checksum";
        script defer type="module" src="/sw.js?hash=checksum" {}
        script defer type="module" src="/static/app.js?hash=checksum" {}
    }
}

fn view_body(model: &Model) -> maud::Markup {
    html! {
        div id=(Id::Glot) class="h-full" {
            (app_layout::app_shell(
                view_content(model),
                None,
                &model.layout_state,
                &model.browser_ctx.current_route(),
            ))

            (search_modal::view(&model.browser_ctx.user_agent, &model.search_modal_state))
        }
    }
}

fn view_content(model: &Model) -> Markup {
    html! {
        div class="flex min-h-full flex-col justify-center py-12 sm:px-6 lg:px-8" {
            div class="sm:mx-auto sm:w-full sm:max-w-md" {
                h2 class="mt-6 text-center text-2xl font-bold leading-9 tracking-tight text-gray-900" {
                    "Login"
                }
            }
            div class="mt-10 sm:mx-auto sm:w-full sm:max-w-[480px]" {
                div class="bg-white px-6 py-12 shadow sm:rounded-lg sm:px-12" {
                    @match model.form_state {
                        FormState::NotSubmitted => {
                            (view_form(model, None))
                        }
                        FormState::Error(ref error) => {
                            (view_form(model, Some(error.clone())))
                        }
                        FormState::Submitted => {
                            p class="text-sm text-gray-500" {
                                "Login link sent! Check your email."
                            }
                        }
                    }
                }
            }
        }
    }
}

fn view_form(model: &Model, error: Option<String>) -> Markup {
    html! {
        form id=(Id::LoginForm) class="space-y-6" {
            div {
                label class="block text-sm font-medium leading-6 text-gray-900" for=(Id::Email) {
                    "Email address"
                }
                div class="mt-2" {
                    input id=(Id::Email) value=(model.email) class="block w-full rounded-md border-0 py-1.5 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-indigo-600 sm:text-sm sm:leading-6" name="email" type="email" autocomplete="email" required;
                }
                p class="mt-2 text-sm text-gray-500" {
                    "A new account will be created if it doesn't exist."
                }
            }
            div {
                button id=(Id::SendLinkButton) class="flex w-full justify-center rounded-md bg-indigo-600 px-3 py-1.5 text-sm font-semibold leading-6 text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600" type="button" {
                    "Send login link"
                }
                @if let Some(ref error) = error {
                    p class="mt-2 text-sm text-gray-500" {
                        (error)
                    }
                }
            }
        }
    }
}
