pub mod item;

pub use item::AdminMediaItem;

use crate::api::media::{delete_media, list_media, upload_media, MediaItem};
use leptos::ev;
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::collections::HashSet;
use wasm_bindgen_futures::JsFuture;
use web_sys::{FileList, HtmlInputElement};

#[component]
pub fn AdminMedia() -> impl IntoView {
    let admin_ctx = crate::context::use_admin_context();
    let (items, set_items) = signal(Vec::<MediaItem>::new());
    let (loading, set_loading) = signal(true);
    let (uploading, set_uploading) = signal(false);
    let (upload_status, set_upload_status) = signal(String::new());
    let (error_msg, set_error_msg) = signal(String::new());
    let (selected_urls, set_selected_urls) = signal(HashSet::<String>::new());
    let (is_deleting, set_is_deleting) = signal(false);
    let (show_options_menu, set_show_options_menu) = signal(false);

    // Filter signals
    let (search_query, set_search_query) = signal(String::new());
    let (filter_type, set_filter_type) = signal("all".to_string());

    // Check token on mount
    Effect::new(move || {
        #[cfg(target_arch = "wasm32")]
        {
            if !admin_ctx.is_admin.get() {
                let navigate = leptos_router::hooks::use_navigate();
                navigate("/admin/login", Default::default());
            }
        }
    });

    let fetch_media = move || {
        set_loading.set(true);
        let t = admin_ctx.token.get();
        if t.is_empty() {
            set_loading.set(false);
            return;
        }

        spawn_local(async move {
            match list_media(t).await {
                Ok(mut res) => {
                    // Sort by name or date if possible, currently keep default order
                    res.reverse(); // Newest first based on naming/timestamp convention
                    set_items.set(res);
                }
                Err(e) => {
                    set_error_msg.set(format!("Error loading media: {}", e));
                }
            }
            set_loading.set(false);
        });
    };

    // Trigger initial fetch when token is set
    Effect::new(move || {
        if !admin_ctx.token.get().is_empty() {
            fetch_media();
        }
    });

    let on_upload = move |ev: ev::Event| {
        let input: HtmlInputElement = event_target(&ev);
        let files: Option<FileList> = input.files();
        if let Some(files) = files {
            let count = files.length();
            if count == 0 {
                return;
            }

            let t = admin_ctx.token.get();
            set_uploading.set(true);
            set_upload_status.set(format!("Uploading {} item(s)...", count));

            let input_clone = input.clone();
            spawn_local(async move {
                let mut success_count = 0;
                let mut errors = Vec::new();

                for i in 0..count {
                    if let Some(file) = files.get(i) {
                        let filename = file.name();
                        let array_buffer_promise = file.array_buffer();
                        match JsFuture::from(array_buffer_promise).await {
                            Ok(array_buffer) => {
                                let uint8_array = js_sys::Uint8Array::new(&array_buffer);
                                let bytes = uint8_array.to_vec();

                                match upload_media(t.clone(), filename.clone(), bytes).await {
                                    Ok(_) => success_count += 1,
                                    Err(e) => errors.push(format!("{}: {}", filename, e)),
                                }
                            }
                            Err(e) => errors.push(format!("{}: {:?}", filename, e)),
                        }
                    }
                }

                set_uploading.set(false);
                input_clone.set_value("");

                if errors.is_empty() {
                    set_upload_status
                        .set(format!("Successfully uploaded {} file(s)!", success_count));
                } else {
                    set_upload_status.set(format!(
                        "Uploaded {} file(s), {} failed: {}",
                        success_count,
                        errors.len(),
                        errors.join(", ")
                    ));
                }
                fetch_media();
            });
        }
    };

    let toggle_selection = move |url: String| {
        set_selected_urls.update(|set| {
            if set.contains(&url) {
                set.remove(&url);
            } else {
                set.insert(url);
            }
        });
    };

    let select_all = move || {
        let all_urls: HashSet<String> = items.get().into_iter().map(|item| item.url).collect();
        set_selected_urls.set(all_urls);
        set_show_options_menu.set(false);
    };

    let deselect_all = move || {
        set_selected_urls.set(HashSet::new());
        set_show_options_menu.set(false);
    };

    let delete_selected = move || {
        let selected = selected_urls.get();
        if selected.is_empty() {
            return;
        }

        #[cfg(target_arch = "wasm32")]
        {
            let count = selected.len();
            let prompt_msg = if count == 1 {
                "Are you sure you want to delete this media item?".to_string()
            } else {
                format!(
                    "Are you sure you want to delete all {} selected media items?",
                    count
                )
            };

            if let Some(win) = web_sys::window() {
                if !win.confirm_with_message(&prompt_msg).unwrap_or(false) {
                    return;
                }
            }
        }

        set_is_deleting.set(true);
        let t = admin_ctx.token.get();

        spawn_local(async move {
            let mut failed = 0;
            for url in selected.iter() {
                let object_name = if let Some(path) =
                    url.strip_prefix("https://storage.googleapis.com/jakewray-portfolio/")
                {
                    path.to_string()
                } else {
                    url.clone()
                };

                if delete_media(t.clone(), object_name).await.is_err() {
                    failed += 1;
                }
            }

            set_is_deleting.set(false);
            set_selected_urls.set(HashSet::new());
            set_show_options_menu.set(false);

            if failed > 0 {
                set_error_msg.set(format!("Failed to delete {} item(s)", failed));
            } else {
                set_upload_status.set("Deleted selected item(s) successfully.".to_string());
            }

            fetch_media();
        });
    };

    // Derived filtered media list
    let filtered_items = move || {
        let query = search_query.get().trim().to_lowercase();
        let ftype = filter_type.get();

        items
            .get()
            .into_iter()
            .filter(|item| {
                let matches_query = if query.is_empty() {
                    true
                } else {
                    item.name.to_lowercase().contains(&query)
                };

                let is_video = item.url.to_lowercase().ends_with(".mp4");
                let matches_type = match ftype.as_str() {
                    "photos" => !is_video,
                    "videos" => is_video,
                    _ => true,
                };

                matches_query && matches_type
            })
            .collect::<Vec<_>>()
    };

    view! {
        <div class="container py-12">
            // Header with back link and title
            <div class="flex flex-col md:flex-row md:items-center justify-between gap-4 mb-8">
                <div>
                    <div class="flex items-center gap-3 mb-2">
                        <a href="/admin/dashboard" class="btn btn-secondary text-xs px-2.5 py-1">
                            "← Back to Dashboard"
                        </a>
                    </div>
                    <h1 class="text-3xl font-bold">"Media Library"</h1>
                    <p class="text-sm text-gray-500 mt-1">"Manage and organize photos and video assets for articles."</p>
                </div>

                // Action Bar (Upload & Dropdown Menu)
                <div class="flex items-center gap-3">
                    <label class="btn btn-primary cursor-pointer flex items-center gap-2">
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
                        </svg>
                        <span>{move || if uploading.get() { "Uploading..." } else { "Upload Media" }}</span>
                        <input
                            type="file"
                            multiple
                            accept="image/*,video/mp4"
                            class="hidden"
                            on:change=on_upload
                            disabled=move || uploading.get()
                        />
                    </label>

                    // Three-dot Options Menu
                    <div class="dropdown-container">
                        <button
                            type="button"
                            class="dropdown-trigger"
                            on:click=move |_| set_show_options_menu.update(|v| *v = !*v)
                            title="Media library options"
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20">
                                <path d="M10 6a2 2 0 110-4 2 2 0 010 4zM10 12a2 2 0 110-4 2 2 0 010 4zM10 18a2 2 0 110-4 2 2 0 010 4z" />
                            </svg>
                        </button>

                        {move || if show_options_menu.get() {
                            Some(view! {
                                <div class="dropdown-menu">
                                    <button type="button" class="dropdown-item" on:click=move |_| select_all()>
                                        <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                                        </svg>
                                        "Select All"
                                    </button>

                                    <button type="button" class="dropdown-item" on:click=move |_| deselect_all() disabled=move || selected_urls.get().is_empty()>
                                        <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z" />
                                        </svg>
                                        "Deselect All"
                                    </button>

                                    <div class="editor-divider" style="width: 100%; margin: 0.25rem 0;"></div>

                                    <button
                                        type="button"
                                        class="dropdown-item danger"
                                        on:click=move |_| delete_selected()
                                        disabled=move || selected_urls.get().is_empty() || is_deleting.get()
                                    >
                                        <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-red-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                                        </svg>
                                        {move || {
                                            let count = selected_urls.get().len();
                                            if count > 0 {
                                                format!("Delete Selected ({})", count)
                                            } else {
                                                "Delete Selected".to_string()
                                            }
                                        }}
                                    </button>
                                </div>
                            })
                        } else { None }}
                    </div>
                </div>
            </div>

            // Filter and Search Toolbar
            <div class="flex flex-col sm:flex-row items-center gap-4 mb-6 pb-4 border-b border-gray-200">
                <div class="w-full sm:w-72 relative">
                    <input
                        type="text"
                        placeholder="Search assets by name..."
                        class="w-full text-sm py-2 px-3 pl-9 rounded-md border border-gray-300 focus:outline-none focus:ring-2 focus:ring-sky-500"
                        prop:value=search_query.get()
                        on:input=move |ev| set_search_query.set(event_target_value(&ev))
                    />
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-gray-400 absolute left-3 top-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                    </svg>
                </div>

                <div class="flex items-center gap-2 w-full sm:w-auto">
                    <select
                        class="text-sm py-2 px-3 rounded-md border border-gray-300 bg-white"
                        prop:value=filter_type.get()
                        on:change=move |ev| set_filter_type.set(event_target_value(&ev))
                    >
                        <option value="all">"All Assets"</option>
                        <option value="photos">"Photos Only"</option>
                        <option value="videos">"Videos Only"</option>
                    </select>
                </div>

                <div class="sm:ml-auto text-xs text-gray-500 font-medium">
                    {move || {
                        let total = items.get().len();
                        let shown = filtered_items().len();
                        let sel = selected_urls.get().len();
                        if sel > 0 {
                            format!("Selected {} of {} assets", sel, total)
                        } else if shown == total {
                            format!("Showing all {} assets", total)
                        } else {
                            format!("Showing {} of {} assets", shown, total)
                        }
                    }}
                </div>
            </div>

            // Status message alerts
            {move || {
                let msg = upload_status.get();
                if !msg.is_empty() {
                    view! {
                        <div class="mb-6 p-3 bg-sky-50 border border-sky-200 text-sky-800 text-sm rounded-lg flex items-center justify-between">
                            <span>{msg}</span>
                            <button type="button" class="text-xs font-bold text-sky-800" on:click=move |_| set_upload_status.set(String::new())>"Dismiss"</button>
                        </div>
                    }.into_any()
                } else {
                    view! { <span class="hidden" /> }.into_any()
                }
            }}

            {move || {
                let err = error_msg.get();
                if !err.is_empty() {
                    view! {
                        <div class="mb-6 p-3 bg-red-50 border border-red-200 text-red-800 text-sm rounded-lg flex items-center justify-between">
                            <span>{err}</span>
                            <button type="button" class="text-xs font-bold text-red-800" on:click=move |_| set_error_msg.set(String::new())>"Dismiss"</button>
                        </div>
                    }.into_any()
                } else {
                    view! { <span class="hidden" /> }.into_any()
                }
            }}

            // Media Grid Content
            {move || {
                if loading.get() {
                    view! {
                        <div class="py-20 text-center text-gray-500">
                            <div class="inline-block animate-spin rounded-full h-8 w-8 border-4 border-sky-500 border-t-transparent mb-3"></div>
                            <p>"Loading media library..."</p>
                        </div>
                    }.into_any()
                } else {
                    let list = filtered_items();
                    if list.is_empty() {
                        view! {
                            <div class="py-16 text-center text-gray-500 bg-gray-50 border border-dashed rounded-xl">
                                <svg xmlns="http://www.w3.org/2000/svg" class="h-12 w-12 mx-auto text-gray-400 mb-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
                                </svg>
                                <p class="font-medium text-gray-700">"No media assets found."</p>
                                <p class="text-xs text-gray-500 mt-1">"Upload photos or videos using the button above."</p>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-4">
                                {list.into_iter().map(|item| {
                                    let item_url = item.url.clone();
                                    let is_sel_check = item_url.clone();
                                    let is_sel = move || selected_urls.get().contains(&is_sel_check);
                                    let on_toggle = move || toggle_selection(item_url.clone());

                                    view! {
                                        <AdminMediaItem
                                            item=item
                                            is_selected=is_sel
                                            on_toggle=on_toggle
                                        />
                                    }
                                }).collect_view()}
                            </div>
                        }.into_any()
                    }
                }
            }}
        </div>
    }
}
