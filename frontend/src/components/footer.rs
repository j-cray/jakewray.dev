use leptos::prelude::*;
use leptos::*;

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer class="py-12 bg-gray-50 border-t border-gray-200 mt-24 text-center">
            <div class="container text-muted text-sm">
                <p>"© 2026 Jake Wray. All rights reserved."</p>
                <div class="flex justify-center gap-4 mt-4">
                    <a href="/contact" class="hover:underline">"Contact"</a>
                    <a href="/admin" class="hover:underline">"Admin"</a>
                </div>
            </div>
        </footer>
    }
}
