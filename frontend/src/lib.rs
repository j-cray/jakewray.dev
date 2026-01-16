mod app;
pub mod components;
pub mod pages;
pub use app::*;
use leptos::prelude::*;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
<<<<<<< HEAD
    leptos::mount::hydrate_body(move || {
        leptos_meta::provide_meta_context();
        view! { <App/> }
    });
=======
    leptos::mount::mount_to_body(App);
>>>>>>> origin/main
}
