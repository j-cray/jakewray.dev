use leptos::prelude::*;

#[component]
pub fn AboutPage() -> impl IntoView {
    view! {
        <div class="container py-12 max-w-2xl">
            <h1 class="text-4xl mb-6">"About Me"</h1>
            <div class="prose">
                <p class="mb-4">
                    "I am a journalist, programmer, and photographer based in [Location]."
                </p>
                <p class="mb-4">
                    "My work focuses on..."
                </p>
            </div>
        </div>
    }
}
