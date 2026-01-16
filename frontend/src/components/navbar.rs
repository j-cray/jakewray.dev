use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use std::rc::Rc;

#[component]
pub fn Navbar() -> impl IntoView {
    let navigate = Rc::new(use_navigate());

    // Helper function to create navigation handler
    let nav_to = move |path: &'static str| {
        let navigate = Rc::clone(&navigate);
        move |_| navigate(path, Default::default())
    };

    view! {
        <header class="p-4 border-b border-gray-200">
            <div class="container mx-auto flex flex-col md:flex-row justify-between items-center gap-4">
                <a href="/" class="text-2xl font-bold font-heading no-underline text-gray-900">
                    "Jake Wray"
                </a>

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
                        "About"
                    </button>
                </div>
            </div>
        </header>
    }
}
