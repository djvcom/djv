pub mod app;
pub mod components;

#[cfg(feature = "ssr")]
pub mod config;

#[cfg(feature = "ssr")]
pub mod db;

#[cfg(feature = "ssr")]
pub mod proxy_headers;

#[cfg(feature = "ssr")]
pub mod sync;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
