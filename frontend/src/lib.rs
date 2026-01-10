use leptos::*; use leptos::prelude::*;

mod app;
pub mod pages;
pub mod components;
pub use app::*;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount_to_body(App);
}
