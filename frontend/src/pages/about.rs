use crate::api::pages::{get_page, save_page, PageContent};
use crate::components::rich_editor::RichTextEditor;
use crate::context::{use_admin_context, AdminAction, AdminActionIcon};
use leptos::prelude::*;
use leptos::task::spawn_local;

pub const ABOUT_EMAIL: &str = "jakewray@mailbox.org";
pub const ABOUT_GITHUB_URL: &str = "https://github.com/j-cray";
pub const ABOUT_GITHUB_LABEL: &str = "github.com/j-cray";

#[component]
pub fn AboutPage() -> impl IntoView {
    let page_resource = Resource::new(|| (), |_| get_page("about".to_string()));
    let admin_ctx = use_admin_context();

    // Edit State
    let (is_editing, set_is_editing) = signal(false);
    let (edit_title, set_edit_title) = signal(String::new());
    let (edit_content, set_edit_content) = signal(String::new());
    let (save_status, set_save_status) = signal(String::new());
    let (is_saving, set_is_saving) = signal(false);

    let turn_on_edit = move |page: &PageContent| {
        set_edit_title.set(page.title.clone());
        set_edit_content.set(page.content_html.clone());
        set_save_status.set(String::new());
        set_is_editing.set(true);
    };

    // Register contextual action with the persistent AdminBar
    Effect::new(move || {
        if admin_ctx.is_admin.get() {
            if is_editing.get() {
                admin_ctx.set_action(AdminAction {
                    label: "Exit Edit Mode".to_string(),
                    icon: AdminActionIcon::Close,
                    href: None,
                    on_click: Some(Callback::new(move |_| {
                        set_is_editing.set(false);
                    })),
                    is_active: true,
                });
            } else {
                admin_ctx.set_action(AdminAction {
                    label: "Edit About Me".to_string(),
                    icon: AdminActionIcon::Edit,
                    href: None,
                    on_click: Some(Callback::new(move |_| {
                        if let Some(Ok(Some(ref page))) = page_resource.get() {
                            turn_on_edit(page);
                        }
                    })),
                    is_active: false,
                });
            }
        }
    });

    on_cleanup(move || {
        admin_ctx.clear_action();
    });

    let on_save = move || {
        let t = admin_ctx.token.get();
        let new_title = edit_title.get();
        let new_content = edit_content.get();

        if new_title.trim().is_empty() {
            set_save_status.set("Page title cannot be empty.".to_string());
            return;
        }

        set_is_saving.set(true);
        set_save_status.set("Saving changes...".to_string());

        let new_page = PageContent {
            slug: "about".to_string(),
            title: new_title,
            content_html: new_content,
            updated_at: None,
        };

        spawn_local(async move {
            match save_page(t, new_page).await {
                Ok(_) => {
                    set_save_status.set("Saved successfully!".to_string());
                    set_is_saving.set(false);
                    set_is_editing.set(false);
                    page_resource.refetch();
                }
                Err(e) => {
                    set_is_saving.set(false);
                    let err_str = e.to_string();
                    if err_str.contains("Invalid token") || err_str.contains("ExpiredSignature") {
                        admin_ctx.logout();
                        set_save_status
                            .set("Save failed: Session expired. Please log in again.".to_string());
                    } else {
                        set_save_status.set(format!("Error: {}", e));
                    }
                }
            }
        });
    };

    view! {
        <div class="py-12">
            {move || {
                if is_editing.get() {
                    view! {
                        <div class="container max-w-4xl bg-white p-8 rounded-lg shadow-sm border border-gray-100 edit-container">
                            <div class="flex items-center justify-between mb-8 pb-4 border-b">
                                <h2 class="text-3xl font-bold text-gray-900">"Editing About Me Page"</h2>
                                <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-semibold bg-sky-100 text-sky-800 border border-sky-200">
                                    "Live Editor"
                                </span>
                            </div>

                            <div class="form-group mb-6">
                                <label class="block font-bold mb-2 text-gray-700">"Page Title"</label>
                                <input
                                    type="text"
                                    class="w-full p-3 border rounded-lg text-xl font-bold text-gray-900 focus:ring-2 focus:ring-sky-500 focus:outline-none"
                                    prop:value=edit_title.get()
                                    on:input=move |ev| set_edit_title.set(event_target_value(&ev))
                                    placeholder="About Me"
                                />
                            </div>

                            <div class="form-group mb-6">
                                <label class="block font-bold mb-2 text-gray-700">"Page Content"</label>
                                <RichTextEditor
                                    value=edit_content
                                    on_change=move |new_val| set_edit_content.set(new_val)
                                />
                            </div>

                            <div class="flex flex-wrap gap-3 items-center mt-8 pt-6 border-t">
                                <button
                                    type="button"
                                    class="btn btn-primary flex items-center gap-2"
                                    on:click=move |_| on_save()
                                    disabled=move || is_saving.get()
                                >
                                    <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                                    </svg>
                                    "Save Changes"
                                </button>
                                <button
                                    type="button"
                                    class="btn btn-secondary"
                                    on:click=move |_| set_is_editing.set(false)
                                    disabled=move || is_saving.get()
                                >
                                    "Cancel"
                                </button>

                                <div class="flex-grow"></div>

                                {move || {
                                    let st = save_status.get();
                                    if !st.is_empty() {
                                        view! { <span class="text-sm font-semibold text-gray-600 bg-gray-100 px-3 py-1.5 rounded">{st}</span> }.into_any()
                                    } else {
                                        view! { <span class="hidden" /> }.into_any()
                                    }
                                }}
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="about-layout">
                            <div class="about-photo-wrapper">
                                <a
                                    href="/jake-lsq-profile.jpg"
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    class="about-photo-link"
                                    aria-label="View full-size photo of Jake Wray in a new tab"
                                >
                                    <img
                                        src="/jake-lsq-profile.jpg"
                                        alt="Jake Wray"
                                        class="about-profile-img"
                                    />
                                </a>
                                <div class="about-pills">
                                    <a
                                        href=format!("mailto:{}", ABOUT_EMAIL)
                                        class="about-pill"
                                        aria-label="Email Jake Wray"
                                    >
                                        <svg
                                            xmlns="http://www.w3.org/2000/svg"
                                            class="about-pill-icon"
                                            viewBox="0 0 20 20"
                                            fill="currentColor"
                                        >
                                            <path d="M2.003 5.884L10 9.882l7.997-3.998A2 2 0 0016 4H4a2 2 0 00-1.997 1.884z" />
                                            <path d="M18 8.118l-8 4-8-4V14a2 2 0 002 2h12a2 2 0 002-2V8.118z" />
                                        </svg>
                                        <span>{ABOUT_EMAIL}</span>
                                    </a>
                                    <a
                                        href=ABOUT_GITHUB_URL
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        class="about-pill"
                                        aria-label="Jake Wray GitHub Profile"
                                    >
                                        <svg
                                            class="about-pill-icon"
                                            viewBox="0 0 24 24"
                                            fill="currentColor"
                                        >
                                            <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z" />
                                        </svg>
                                        <span>{ABOUT_GITHUB_LABEL}</span>
                                    </a>
                                </div>
                            </div>
                            <div class="about-container bg-white p-8 rounded-lg shadow-sm border border-gray-100">
                                <Suspense fallback=move || view! { <p class="text-gray-500 py-8 text-center">"Loading..."</p> }>
                                    {move || {
                                        page_resource.get().map(|res| {
                                            match res {
                                                Ok(Some(page)) => {
                                                    let title = page.title.clone();
                                                    let content = page.content_html.clone();
                                                    view! {
                                                        <div>
                                                            <h1 class="text-4xl mb-6 font-bold text-gray-900 border-b border-gray-100 pb-4">{title}</h1>
                                                            <div class="about-content prose prose-lg text-gray-700 leading-relaxed" inner_html=content></div>
                                                        </div>
                                                    }.into_any()
                                                }
                                                Ok(None) => {
                                                    view! {
                                                        <div class="text-center py-8">
                                                            <h1 class="text-3xl font-bold mb-4">"About Me"</h1>
                                                            <p class="text-gray-600">"Page content not found."</p>
                                                        </div>
                                                    }.into_any()
                                                }
                                                Err(e) => {
                                                    view! {
                                                        <p class="text-red-600 py-4">"Error loading page: " {e.to_string()}</p>
                                                    }.into_any()
                                                }
                                            }
                                        })
                                    }}
                                </Suspense>
                            </div>
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_about_contact_info() {
        assert_eq!(ABOUT_EMAIL, "jakewray@mailbox.org");
        assert!(ABOUT_EMAIL.contains('@'));
        assert_eq!(ABOUT_GITHUB_URL, "https://github.com/j-cray");
        assert!(ABOUT_GITHUB_URL.starts_with("https://github.com/"));
        assert_eq!(ABOUT_GITHUB_LABEL, "github.com/j-cray");
    }
}
