use leptonic::components::button::Button;
use leptonic::prelude::*;
use leptos::prelude::*;
use leptos::*;

#[component]
pub fn Navbar() -> impl IntoView {
    let navigate = leptos_router::hooks::use_navigate();

    let nav_btn = {
        let navigate = navigate.clone();
        move |text: &'static str, path: &'static str| {
            let navigate = navigate.clone();
            view! {
                <button on:click=move |_| navigate(path, Default::default()) style="margin: 0 0.5em; padding: 0.5em 1em; cursor: pointer;">
                    {text}
                </button>
            }
        }
    };

    view! {
        <div style="padding: 1em; border-bottom: 1px solid #eee;">
            <div style="display: flex; justify-content: space-between; align-items: center;">
                <h3 style="margin: 0">
                    <a href="/" style="text-decoration: none; color: inherit;">"Jake Wray"</a>
                </h3>

                <div style="display: flex; gap: 0.5em;">
                    {nav_btn("Journalism", "/journalism")}
                    {nav_btn("Personal", "/personal")}
                    {nav_btn("Writing", "/creative-writing")}
                    {nav_btn("Music", "/music")}
                    {nav_btn("Art", "/visual-art")}
                    {nav_btn("Code", "/programming")}

                    <button on:click=move |_| navigate.clone()("/about", Default::default()) style="margin: 0 0.5em; padding: 0.5em 1em; cursor: pointer; font-weight: bold;">
                        "About"
                    </button>
                </div>
            </div>
        </div>
    }
}
