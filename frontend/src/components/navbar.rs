use leptos::prelude::*;
use leptos::*;

#[component]
pub fn Navbar() -> impl IntoView {
    let navigate = leptos_router::hooks::use_navigate();

    view! {
        <header class="p-4 border-b border-gray-200">
            <div class="container mx-auto flex flex-col md:flex-row justify-between items-center gap-4">
                <a href="/" class="text-2xl font-bold font-heading no-underline text-gray-900">
                    "Jake Wray"
                </a>

                <div class="flex gap-2 flex-wrap justify-center">
                    <button class="px-4 py-2 rounded-md hover:bg-gray-100 transition-colors" on:click={
                        let navigate = navigate.clone();
                        move |_| { navigate("/journalism", Default::default()); }
                    }>
                        "Journalism"
                    </button>
                    <button class="px-4 py-2 rounded-md hover:bg-gray-100 transition-colors" on:click={
                        let navigate = navigate.clone();
                        move |_| { navigate("/personal", Default::default()); }
                    }>
                        "Personal"
                    </button>
                    <button class="px-4 py-2 rounded-md hover:bg-gray-100 transition-colors" on:click={
                        let navigate = navigate.clone();
                        move |_| { navigate("/creative-writing", Default::default()); }
                    }>
                        "Writing"
                    </button>
                    <button class="px-4 py-2 rounded-md hover:bg-gray-100 transition-colors" on:click={
                        let navigate = navigate.clone();
                        move |_| { navigate("/music", Default::default()); }
                    }>
                        "Music"
                    </button>
                    <button class="px-4 py-2 rounded-md hover:bg-gray-100 transition-colors" on:click={
                        let navigate = navigate.clone();
                        move |_| { navigate("/visual-art", Default::default()); }
                    }>
                        "Art"
                    </button>
                    <button class="px-4 py-2 rounded-md hover:bg-gray-100 transition-colors" on:click={
                        let navigate = navigate.clone();
                        move |_| { navigate("/programming", Default::default()); }
                    }>
                        "Code"
                    </button>
                    <div class="w-px h-6 bg-gray-300 mx-2 hidden md:block"></div>
                    <button class="px-4 py-2 rounded-md bg-black text-white hover:bg-gray-800 transition-colors" on:click={
                        let navigate = navigate.clone();
                        move |_| { navigate("/about", Default::default()); }
                    }>
                        "About"
                    </button>
                </div>
            </div>
        </header>
    }
}
