use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn Navbar() -> impl IntoView {
    view! {
        <header class="site-header">
            <div class="container nav-container">
                <a href="/" class="site-brand">
                    "Jake Wray"
                </a>

                <nav class="nav-links">
                    <A href="/code" class="nav-link" active_class="active">"Code"</A>
                    <A href="/blog" class="nav-link" active_class="active">"Blog"</A>
                    <A href="/journalism" class="nav-link" active_class="active">"Journalism"</A>
                    <A href="/about" class="nav-link" active_class="active">"About Me"</A>
                </nav>
            </div>
        </header>
    }
}
