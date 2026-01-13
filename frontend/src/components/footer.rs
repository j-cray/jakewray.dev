use leptonic::prelude::*;
use leptos::prelude::*;

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer style="padding: 3em; margin-top: 3em; text-align: center;">
            <div style="display: flex; flex-direction: column; gap: 1em;">
                <p>"© 2026 Jake Wray. All rights reserved."</p>
                <div style="display: flex; justify-content: center; gap: 1em;">
                    <a href="/contact">"Contact"</a>
                    <a href="/admin">"Admin"</a>
                </div>
            </div>
        </footer>
    }
}
