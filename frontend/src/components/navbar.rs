use icondata::BsList;
use leptos::prelude::*;
<<<<<<< HEAD

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
=======
use leptos_router::hooks::use_navigate;
use std::rc::Rc;

#[component]
pub fn Navbar() -> impl IntoView {
    let navigate = Rc::new(use_navigate());

    // Helper function to create navigation handler
    let nav_to = move |path: &'static str| {
        let navigate = Rc::clone(&navigate);
        move |_| navigate(path, Default::default())
>>>>>>> origin/main
    };

    view! {
        <header class="glass sticky top-0 z-50">
            <div class="flex items-center justify-between w-full h-full px-6 py-3">
                <h3 class="m-0 cursor-pointer text-2xl" on:click={
                    let navigate = navigate.clone();
                    move |_| navigate("/", Default::default())
                }>
                    <span class="text-gradient font-bold tracking-tight">"JAKE WRAY"</span>
                </h3>

<<<<<<< HEAD
                // Desktop Menu
                <div class="hidden md:flex gap-4 items-center">
                    {nav_btn("Journalism", "/journalism")}
                    {nav_btn("Personal", "/personal")}
                    {nav_btn("Writing", "/creative-writing")}
                    {nav_btn("Music", "/music")}
                    {nav_btn("Art", "/visual-art")}
                    {nav_btn("Code", "/programming")}

                    <button on:click={
                        let navigate = navigate.clone();
                        move |_| navigate("/about", Default::default())
                    } class="btn-primary">
=======
                <div class="flex gap-2 flex-wrap justify-center">
                    <button class="px-4 py-2 rounded-md hover:bg-gray-100 transition-colors" on:click=nav_to("/journalism")>
                        "Journalism"
                    </button>
                    <button class="px-4 py-2 rounded-md hover:bg-gray-100 transition-colors" on:click=nav_to("/personal")>
                        "Personal"
                    </button>
                    <button class="px-4 py-2 rounded-md hover:bg-gray-100 transition-colors" on:click=nav_to("/creative-writing")>
                        "Writing"
                    </button>
                    <button class="px-4 py-2 rounded-md hover:bg-gray-100 transition-colors" on:click=nav_to("/music")>
                        "Music"
                    </button>
                    <button class="px-4 py-2 rounded-md hover:bg-gray-100 transition-colors" on:click=nav_to("/visual-art")>
                        "Art"
                    </button>
                    <button class="px-4 py-2 rounded-md hover:bg-gray-100 transition-colors" on:click=nav_to("/programming")>
                        "Code"
                    </button>
                    <div class="w-px h-6 bg-gray-300 mx-2 hidden md:block"></div>
                    <button class="px-4 py-2 rounded-md bg-black text-white hover:bg-gray-800 transition-colors" on:click=nav_to("/about")>
>>>>>>> origin/main
                        "About"
                    </button>
                </div>

                <button on:click=move |_| set_drawer_open.set(true) class="md:hidden text-2xl text-white hover:text-brand transition">
                    <span class="sr-only">"Open menu"</span>
                    "☰"
                </button>
            </div>
        </header>

        {move || is_drawer_open.get().then(|| view! {
             <div class="fixed inset-0 z-[100] flex justify-end">
                // Backdrop
                <div
                    class="absolute inset-0 bg-black/60 backdrop-blur-sm transition-opacity"
                    on:click=move |_| set_drawer_open.set(false)
                ></div>

                // Drawer
                <div class="relative w-80 h-full bg-[#1a1a1a] shadow-2xl border-l border-white/10 p-6 flex flex-col gap-6 transform transition-transform animate-in slide-in-from-right duration-300">
                    <div class="flex justify-between items-center mb-4">
                        <h3 class="text-xl font-bold text-white">"Menu"</h3>
                        <button on:click=move |_| set_drawer_open.set(false) class="text-gray-400 hover:text-white">
                            "✕"
                        </button>
                    </div>

                    <div class="flex flex-col gap-3">
                        {nav_btn("Journalism", "/journalism")}
                        {nav_btn("Personal", "/personal")}
                        {nav_btn("Writing", "/creative-writing")}
                        {nav_btn("Music", "/music")}
                        {nav_btn("Art", "/visual-art")}
                        {nav_btn("Code", "/programming")}
                        {nav_btn("About", "/about")}
                    </div>

                    <button on:click=move |_| set_drawer_open.set(false) class="btn-secondary mt-auto w-full">
                        "Close Menu"
                    </button>
                </div>
            </div>
        })}
    }
}
