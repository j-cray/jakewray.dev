use crate::api::pages::{get_page, save_page, PageContent};
use crate::components::rich_editor::RichTextEditor;
use leptos::prelude::*;
use leptos::task::spawn_local;

#[component]
pub fn AboutPage() -> impl IntoView {
    let page_resource = Resource::new(|| (), |_| get_page("about".to_string()));

    // Auth State
    let (is_admin, set_is_admin) = signal(false);
    let (token, set_token) = signal(String::new());

    Effect::new(move || {
        #[cfg(target_arch = "wasm32")]
        {
            if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
                if let Ok(Some(t)) = storage.get_item("admin_token") {
                    if !t.is_empty() {
                        if shared::auth::is_token_expired(&t) {
                            let _ = storage.remove_item("admin_token");
                            set_is_admin.set(false);
                            set_token.set(String::new());
                        } else {
                            set_token.set(t);
                            set_is_admin.set(true);
                        }
                    }
                }
            }
        }
    });

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

    let on_save = move || {
        let t = token.get();
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
                        #[cfg(target_arch = "wasm32")]
                        if let Some(window) = web_sys::window() {
                            if let Ok(Some(storage)) = window.local_storage() {
                                let _ = storage.remove_item("admin_token");
                            }
                        }
                        set_is_admin.set(false);
                        set_token.set(String::new());
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
                        <div class="about-container container max-w-2xl bg-white p-8 rounded-lg shadow-sm border border-gray-100">
                            {move || {
                                is_admin.get().then(|| {
                                    let page_data = page_resource.get().and_then(|r| r.ok()).flatten();
                                    view! {
                                        <div class="mb-6 p-4 bg-gray-50 border rounded-lg flex items-center justify-between">
                                            <div class="flex items-center gap-2">
                                                <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-semibold bg-amber-100 text-amber-800 border border-amber-200">
                                                    "Admin Mode"
                                                </span>
                                                <span class="text-xs text-gray-500 font-medium">"You can edit the content on this page"</span>
                                            </div>
                                            <button
                                                type="button"
                                                class="btn btn-sm btn-primary flex items-center gap-1.5"
                                                on:click=move |_| {
                                                    if let Some(ref p) = page_data {
                                                        turn_on_edit(p);
                                                    }
                                                }
                                            >
                                                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                                                </svg>
                                                "Edit About Me"
                                            </button>
                                        </div>
                                    }
                                })
                            }}

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
                    }.into_any()
                }
            }}
        </div>
    }
}
