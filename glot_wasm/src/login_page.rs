use glot_core::common::browser_context::JsBrowserContext;
use glot_core::page::login_page;
use poly::page::wasm;
use poly::page::Page;
use poly_macro::impl_wasm_page;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct LoginPage(login_page::LoginPage);

impl_wasm_page!(LoginPage);

#[wasm_bindgen(js_name = loginPage)]
pub fn new(js_browser_ctx: JsValue) -> Result<LoginPage, JsValue> {
    let browser_ctx: JsBrowserContext = wasm::decode_js_value(js_browser_ctx)
        .map_err(|err| format!("Failed to decode browser context: {}", err))?;

    Ok(LoginPage(login_page::LoginPage {
        browser_ctx: browser_ctx.into_browser_context(),
    }))
}
