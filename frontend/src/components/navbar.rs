use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_location;

#[component]
pub fn Navbar() -> impl IntoView {
    let location = use_location();
    let is_active = move |path: &'static str| {
        let location = location.clone();
        move || {
            if location.pathname.get().starts_with(path) {
                "nav-link active"
            } else {
                "nav-link"
            }
        }
    };

    view! {
        <header class="site-header">
            <div class="container nav-container">
                <A href="/" attr:class="site-brand">"Jake Wray"</A>
                <nav class="nav-links"><A href="/code" attr:class=is_active("/code")>"Code"</A><A href="/blog" attr:class=is_active("/blog")>"Blog"</A><A href="/journalism" attr:class=is_active("/journalism")>"Journalism"</A><A href="/about" attr:class=is_active("/about")>"About Me"</A></nav>
            </div>
        </header>
    }
}
