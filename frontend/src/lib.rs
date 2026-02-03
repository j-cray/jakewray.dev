mod app;
pub mod components;
pub mod api;
pub mod data;
pub mod pages;
pub use app::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    web_sys::console::log_1(&"Hydration started".into());
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
