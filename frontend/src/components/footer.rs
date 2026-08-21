use leptos::prelude::*;

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer class="py-6 bg-gray-50 border-t border-gray-200 text-center">
            <div class="container text-muted text-sm">
                <p>"© 2026 Jake Wray. All rights reserved."</p>
            </div>
        </footer>
    }
}
