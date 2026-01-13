use leptonic::prelude::*;
use leptos::prelude::*;

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer style="padding: 3em 0; margin-top: auto; text-align: center; border-top: 1px solid var(--glass-border);">
            <div style="display: flex; flex-direction: column; gap: 1em; opacity: 0.7;">
                <p style="font-size: 0.9em;">
                    "© 2026 Jake Wray. Licensed under "
                    <a href="https://creativecommons.org/licenses/by-nc/4.0/" target="_blank" rel="noopener noreferrer" style="color: var(--brand-color);">"CC BY-NC 4.0"</a>
                    "."
                </p>
                <div style="display: flex; justify-content: center; gap: 1.5em;">
                    <a href="/contact" style="color: inherit; text-decoration: none; hover: color: var(--brand-color);">"Contact"</a>
                    <a href="https://github.com/j-cray" target="_blank" style="color: inherit; text-decoration: none;">"GitHub"</a>
                    <a href="/admin" style="color: inherit; text-decoration: none;">"Admin"</a>
                </div>
            </div>
        </footer>
    }
}
