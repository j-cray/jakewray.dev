use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn Navbar() -> impl IntoView {
    view! {
        <header class="site-header">
            <div class="container nav-container">
                <A href="/" class="site-brand">
                    "Jake Wray"
                </A>

                <nav class="nav-links">
                    <A class="nav-link" href="/journalism">"Journalism"</A>
                    <A class="nav-link" href="/personal">"Personal"</A>
                    <A class="nav-link" href="/personal/blog">"Blog"</A>
                    <A class="nav-link" href="/programming">"Code"</A>
                    <A class="nav-link nav-link-primary" href="/about">"About"</A>
                </nav>
            </div>
        </header>
    }
}
