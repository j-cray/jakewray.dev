use leptos::prelude::*;
use leptos::*;
use thaw::*;

#[component]
pub fn Navbar() -> impl IntoView {
    let navigate = leptos_router::hooks::use_navigate();

    view! {
        <LayoutHeader class="p-4 border-b border-gray-200">
            <div class="container mx-auto flex flex-col md:flex-row justify-between items-center gap-4">
                <a href="/" class="text-2xl font-bold font-heading no-underline text-gray-900">
                    "Jake Wray"
                </a>

                <div class="flex gap-2 flex-wrap justify-center">
                    <Button on_click={
                        let navigate = navigate.clone();
                        move |_| { navigate("/journalism", Default::default()); }
                    }>
                        "Journalism"
                    </Button>
                    <Button on_click={
                        let navigate = navigate.clone();
                        move |_| { navigate("/personal", Default::default()); }
                    }>
                        "Personal"
                    </Button>
                    <Button on_click={
                        let navigate = navigate.clone();
                        move |_| { navigate("/creative-writing", Default::default()); }
                    }>
                        "Writing"
                    </Button>
                    <Button on_click={
                        let navigate = navigate.clone();
                        move |_| { navigate("/music", Default::default()); }
                    }>
                        "Music"
                    </Button>
                    <Button on_click={
                        let navigate = navigate.clone();
                        move |_| { navigate("/visual-art", Default::default()); }
                    }>
                        "Art"
                    </Button>
                    <Button on_click={
                        let navigate = navigate.clone();
                        move |_| { navigate("/programming", Default::default()); }
                    }>
                        "Code"
                    </Button>
                    <div class="w-px h-6 bg-gray-300 mx-2 hidden md:block"></div>
                    <Button on_click={
                        let navigate = navigate.clone();
                        move |_| { navigate("/about", Default::default()); }
                    }>
                        "About"
                    </Button>
                </div>
            </div>
        </LayoutHeader>
    }
}
