use leptos::prelude::*;

#[component]
pub fn Navbar() -> impl IntoView {
    view! {
        <header class="site-header">
            <div class="container nav-container">
                <a href="/" class="site-brand">
                    "Jake Wray"
                </a>

                <nav class="nav-links">
                    <a class="nav-link" href="/code">"Code"</a>
                    <a class="nav-link" href="/journalism">"Journalism"</a>
                    <a class="nav-link" href="/blog">"Blog"</a>
                </nav>
            </div>
        </header>
    }
}
