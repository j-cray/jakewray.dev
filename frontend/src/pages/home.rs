use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <div class="container home-hero">
            <header class="hero">
                <h1 class="hero-title">"Jake Wray"</h1>
                <p class="hero-subtitle">
                    "A work in progress (me and the website)"
                </p>
            </header>

            <div class="card-grid">
                <div class="card">
                    <h3 class="text-xl font-bold">"Latest Articles"</h3>
                    <p class="text-muted">"Coming soon..."</p>
                </div>
                <div class="card">
                    <h3 class="text-xl font-bold">"Recent Projects"</h3>
                    <p class="text-muted">"Coming soon..."</p>
                </div>
                <div class="card">
                    <h3 class="text-xl font-bold">"Visuals"</h3>
                    <p class="text-muted">"Coming soon..."</p>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn NotFound() -> impl IntoView {
    view! {
        <div class="container py-24 text-center">
            <h1 class="text-4xl mb-4">"404"</h1>
            <p>"Page not found."</p>
        </div>
    }
}

#[component]
pub fn AdminRedirect() -> impl IntoView {
    let navigate = use_navigate();
    leptos::prelude::Effect::new(move || {
        navigate("/admin/login", Default::default());
    });
}
