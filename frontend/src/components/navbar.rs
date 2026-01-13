use icondata::BsList;
use leptos::prelude::*;

// use leptonic... imports removed

#[component]
pub fn Navbar() -> impl IntoView {
    let navigate = leptos_router::hooks::use_navigate();
    let (is_drawer_open, set_drawer_open) = create_signal(false);

    let nav_btn = {
        let navigate = navigate.clone();
        move |text: &'static str, path: &'static str| {
            let navigate = navigate.clone();
            view! {
                <button on:click=move |_| { navigate(path, Default::default()); set_drawer_open.set(false); } style="font-weight: 500;" class="btn-flat">
                    {text}
                </button>
            }
        }
    };

    view! {
        <header class="glass sticky top-0 z-50">
            <div class="flex items-center justify-between w-full h-full px-4">
                <h3 class="m-0 cursor-pointer" on:click={
                    let navigate = navigate.clone();
                    move |_| navigate("/", Default::default())
                }>
                    <span class="text-gradient" style="font-weight: bold;">"JAKE WRAY"</span>
                </h3>

                // Desktop Menu
                <div class="hidden md:flex gap-2">
                    {nav_btn("Journalism", "/journalism")}
                    {nav_btn("Personal", "/personal")}
                    {nav_btn("Writing", "/creative-writing")}
                    {nav_btn("Music", "/music")}
                    {nav_btn("Art", "/visual-art")}
                    {nav_btn("Code", "/programming")}

                    <button on:click={
                        let navigate = navigate.clone();
                        move |_| navigate("/about", Default::default())
                    } class="btn-filled">
                        "About"
                    </button>
                </div>

                <button on:click=move |_| set_drawer_open.set(true) class="md:hidden text-2xl font-bold">
                    "☰"
                </button>
            </div>
        </header>

        {move || is_drawer_open.get().then(|| view! {
            <div style="position: fixed; top: 0; right: 0; height: 100vh; width: 300px; background: #1a1a1a; padding: 2em; z-index: 100; border-left: 1px solid #333;" class="shadow-xl">
                 <div class="flex flex-col gap-6">
                    <h3 class="text-xl font-bold">"Menu"</h3>
                    {nav_btn("Journalism", "/journalism")}
                    {nav_btn("Personal", "/personal")}
                    {nav_btn("Writing", "/creative-writing")}
                    {nav_btn("Music", "/music")}
                    {nav_btn("Art", "/visual-art")}
                    {nav_btn("Code", "/programming")}
                    {nav_btn("About", "/about")}
                    <button on:click=move |_| set_drawer_open.set(false) class="btn-outlined">
                        "Close"
                    </button>
                </div>
            </div>
        })}
    }
}
