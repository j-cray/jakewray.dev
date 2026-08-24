use super::types::PinnedRepo;
use leptos::prelude::*;

#[component]
pub fn ProfileHeader() -> impl IntoView {
    view! {
        <div class="code-profile-header card">
            <div class="profile-header-content">
                <div class="profile-avatar-wrapper">
                    <img
                        src="https://avatars.githubusercontent.com/u/150755225?v=4"
                        alt="Jake Wray GitHub Avatar"
                        class="profile-avatar"
                        loading="lazy"
                    />
                    <div class="profile-status-indicator" title="Active on GitHub" />
                </div>
                <div class="profile-info">
                    <div class="profile-title-row">
                        <h2 class="profile-name">"Jake Wray"</h2>
                        <a
                            href="https://github.com/j-cray"
                            target="_blank"
                            rel="noopener noreferrer"
                            class="profile-handle"
                        >
                            "@j-cray"
                        </a>
                    </div>
                    <p class="profile-bio">
                        "vibe coder, devops enthusiast, freelance journalist"
                    </p>
                    <div class="profile-badges">
                        <span class="badge badge-subtle">
                            <svg xmlns="http://www.w3.org/2000/svg" class="icon-sm" viewBox="0 0 20 20" fill="currentColor">
                                <path fill-rule="evenodd" d="M5.05 4.05a7 7 0 119.9 9.9L10 18.9l-4.95-4.95a7 7 0 010-9.9zM10 11a2 2 0 100-4 2 2 0 000 4z" clip-rule="evenodd" />
                            </svg>
                            "Northern BC, Canada"
                        </span>
                        <span class="badge badge-subtle">
                            <svg xmlns="http://www.w3.org/2000/svg" class="icon-sm" viewBox="0 0 20 20" fill="currentColor">
                                <path fill-rule="evenodd" d="M12.316 3.051a1 1 0 01.633 1.265l-4 12a1 1 0 11-1.898-.632l4-12a1 1 0 011.265-.633zM5.707 6.293a1 1 0 010 1.414L2.414 11l3.293 3.293a1 1 0 01-1.414 1.414l-4-4a1 1 0 010-1.414l4-4a1 1 0 011.414 0zm8.586 0a1 1 0 011.414 0l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414-1.414L17.586 11l-3.293-3.293a1 1 0 010-1.414z" clip-rule="evenodd" />
                            </svg>
                            "Rust • Nix • Web"
                        </span>
                    </div>
                </div>
                <div class="profile-actions">
                    <a
                        href="https://github.com/j-cray"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="btn btn-primary github-profile-btn"
                        on:click=move |_| {
                            crate::utils::analytics::track_outbound_click("https://github.com/j-cray", Some("GitHub Profile"), false);
                        }
                    >
                        <svg class="icon-github" viewBox="0 0 24 24" fill="currentColor">
                            <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z" />
                        </svg>
                        "View on GitHub"
                    </a>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn CommitGraphWidget() -> impl IntoView {
    view! {
        <section class="commit-graph-section card">
            <div class="commit-graph-container">
                <img
                    src="https://ghchart.rshah.org/059669/j-cray"
                    alt="j-cray GitHub Contribution Graph"
                    class="commit-graph-img"
                    loading="lazy"
                />
            </div>
        </section>
    }
}

#[component]
pub fn RepoCard(repo: PinnedRepo) -> impl IntoView {
    let repo_url = repo.url.clone();
    let display_name = repo.display_name();
    let is_fork = repo.is_fork;
    let description = repo.description.clone();
    let language = repo.language.clone();
    let language_color = repo.language_color.clone();
    let tags = repo.tags.clone();

    let repo_url_click1 = repo_url.clone();
    let display_name_click1 = display_name.clone();
    let repo_url_click2 = repo_url.clone();
    let display_name_click2 = display_name.clone();

    view! {
        <article class="repo-card card">
            <div class="repo-card-header">
                <div class="repo-title-wrapper">
                    <svg class="icon-repo" viewBox="0 0 16 16" fill="currentColor">
                        <path fill-rule="evenodd" d="M2 2.5A2.5 2.5 0 014.5 0h8.75a.75.75 0 01.75.75v12.5a.75.75 0 01-.75.75h-2.5a.75.75 0 110-1.5h1.75v-2h-8a1 1 0 00-.714 1.7.75.75 0 01-1.072 1.05A2.495 2.495 0 012 11.5v-9zm10.5-1V9h-8c-.356 0-.694.074-1 .208V2.5a1 1 0 011-1h8zM5 12.25v3.25a.25.25 0 00.4.2l1.45-1.087a.25.25 0 01.3 0L8.6 15.7a.25.25 0 00.4-.2v-3.25a.25.25 0 00-.25-.25h-3.5a.25.25 0 00-.25.25z" />
                    </svg>
                    <a
                        href=repo_url.clone()
                        target="_blank"
                        rel="noopener noreferrer"
                        class="repo-name-link"
                        on:click=move |_| {
                            crate::utils::analytics::track_outbound_click(&repo_url_click1, Some(&display_name_click1), false);
                        }
                    >
                        {display_name}
                    </a>
                </div>
                <div class="repo-badge-wrapper">
                    {if is_fork {
                        view! { <span class="badge badge-fork">"Fork"</span> }.into_any()
                    } else {
                        view! { <span class="badge badge-public">"Public"</span> }.into_any()
                    }}
                </div>
            </div>

            <p class="repo-description">{description}</p>

            <div class="repo-card-footer">
                <div class="repo-meta-left">
                    <span class="repo-language">
                        <span
                            class="language-color-dot"
                            style:background-color=language_color
                        />
                        {language}
                    </span>
                    <div class="repo-tags">
                        {tags.into_iter().map(|tag| {
                            view! {
                                <span class="tag-chip">{tag}</span>
                            }
                        }).collect_view()}
                    </div>
                </div>

                <a
                    href=repo_url
                    target="_blank"
                    rel="noopener noreferrer"
                    class="repo-external-link"
                    title="View repository on GitHub"
                    on:click=move |_| {
                        crate::utils::analytics::track_outbound_click(&repo_url_click2, Some(&display_name_click2), false);
                    }
                >
                    <svg class="icon-sm" viewBox="0 0 20 20" fill="currentColor">
                        <path d="M11 3a1 1 0 100 2h2.586l-6.293 6.293a1 1 0 101.414 1.414L15 6.414V9a1 1 0 102 0V4a1 1 0 00-1-1h-5z" />
                        <path d="M5 5a2 2 0 00-2 2v8a2 2 0 002 2h8a2 2 0 002-2v-3a1 1 0 10-2 0v3H5V7h3a1 1 0 000-2H5z" />
                    </svg>
                </a>
            </div>
        </article>
    }
}

#[component]
pub fn PinnedReposGrid(repos: Vec<PinnedRepo>) -> impl IntoView {
    view! {
        <section class="pinned-repos-section">
            <div class="section-header">
                <div>
                    <h3 class="section-title">"Pinned Repositories"</h3>
                    <p class="section-subtitle">"Featured open-source projects, CLI tools, and libraries"</p>
                </div>
            </div>
            <div class="repos-grid">
                {repos.into_iter().map(|repo| {
                    view! {
                        <RepoCard repo=repo />
                    }
                }).collect_view()}
            </div>
        </section>
    }
}
