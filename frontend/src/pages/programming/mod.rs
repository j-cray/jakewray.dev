pub mod components;
pub mod types;

#[cfg(test)]
pub mod tests;

use components::{CommitGraphWidget, PinnedReposGrid, ProfileHeader};
use leptos::prelude::*;
use types::get_pinned_repos;

#[component]
pub fn ProgrammingPage() -> impl IntoView {
    let pinned_repos = get_pinned_repos();

    view! {
        <div class="code-page container py-8">
            <div class="code-page-header mb-8">
                <h1 class="page-title text-4xl font-bold mb-2">"Code"</h1>
                <p class="text-muted text-lg">
                    "A showcase of my software projects, systems tooling, and GitHub activity."
                </p>
            </div>

            <ProfileHeader />

            <CommitGraphWidget />

            <PinnedReposGrid repos=pinned_repos />
        </div>
    }
}
