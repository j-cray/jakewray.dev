use crate::api::articles::{delete_media, list_media, upload_media, MediaItem};
use leptos::ev;
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::collections::HashSet;
use wasm_bindgen_futures::JsFuture;
use web_sys::{FileList, HtmlInputElement};

#[component]
fn AdminMediaItem(
    item: MediaItem,
    is_selected: impl Fn() -> bool + Send + Sync + 'static,
    on_toggle: impl Fn() + Send + Sync + 'static,
) -> impl IntoView {
    // Default fallback aspect ratio of 3:2 (1.5)
    let (aspect_ratio, set_aspect_ratio) = signal(1.5);

    let url = item.url.clone();
    let name = item.name.clone();
    let name_title = name.clone();
    let is_video = url.to_lowercase().ends_with(".mp4");

    let on_img_load = move |ev: ev::Event| {
        let img: web_sys::HtmlImageElement = event_target(&ev);
        let w = img.natural_width();
        let h = img.natural_height();
        if h > 0 {
            set_aspect_ratio.set(w as f64 / h as f64);
        }
    };

    let on_video_load = move |ev: ev::Event| {
        let video: web_sys::HtmlVideoElement = event_target(&ev);
        let w = video.video_width();
        let h = video.video_height();
        if h > 0 {
            set_aspect_ratio.set(w as f64 / h as f64);
        }
    };

    let is_sel = std::sync::Arc::new(is_selected);
    let is_sel_checkbox = is_sel.clone();
    let is_sel_class = is_sel;

    let on_toggle = std::sync::Arc::new(on_toggle);
    let on_toggle_card = on_toggle.clone();
    let on_toggle_checkbox = on_toggle;

    view! {
        <div
            class="relative group flex flex-col gap-2 cursor-pointer"
            class:is-selected=move || is_sel_class()
            on:click=move |_| on_toggle_card()
        >
            // Aspect ratio thumbnail wrapper
            <div
                class="media-thumbnail bg-white"
                style=move || format!("aspect-ratio: {};", aspect_ratio.get())
            >
                // Circle checkbox overlay
                <div
                    class="selection-checkbox"
                    class:is-selected=move || is_sel_checkbox()
                    on:click=move |ev| {
                        ev.stop_propagation();
                        on_toggle_checkbox();
                    }
                >
                    <svg viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M4.5 12.75l6 6 9-13.5" />
                    </svg>
                </div>

                // Photo / video element
                {if is_video {
                    view! {
                        <video
                            src=url.clone()
                            muted
                            playsinline
                            preload="metadata"
                            on:loadedmetadata=on_video_load
                        />
                    }.into_any()
                } else {
                    view! {
                        <img
                            src=url.clone()
                            alt=name.clone()
                            on:load=on_img_load
                        />
                    }.into_any()
                }}
            </div>

            // Asset Details (Title/Name)
            <div class="pt-1 px-1 mt-auto">
                <span class="text-xs font-semibold truncate text-gray-700 block" title=name_title>{name}</span>
            </div>
        </div>
    }
}

#[component]
pub fn AdminMedia() -> impl IntoView {
    let (items, set_items) = signal(Vec::<MediaItem>::new());
    let (selected_urls, set_selected_urls) = signal(HashSet::<String>::new());
    let (loading, set_loading) = signal(true);
    let (uploading, set_uploading) = signal(false);
    let (menu_open, set_menu_open) = signal(false);
    let (error_msg, set_error_msg) = signal(String::new());
    let (success_msg, set_success_msg) = signal(String::new());
    let (token, set_token) = signal(String::new());
    let _ = set_token;

    let go_login = move || {
        #[cfg(target_arch = "wasm32")]
        {
            (leptos_router::hooks::use_navigate())("/admin/login", Default::default());
        }
    };

    // Check auth on load
    Effect::new(move || {
        #[cfg(target_arch = "wasm32")]
        {
            let window = web_sys::window().unwrap();
            let local_storage = window.local_storage().unwrap().unwrap();
            if let Ok(Some(t)) = local_storage.get_item("admin_token") {
                if !t.is_empty() && !shared::auth::is_token_expired(&t) {
                    set_token.set(t);
                    return;
                }
                let _ = local_storage.remove_item("admin_token");
            }
            go_login();
        }
    });

    let fetch_media = move |is_initial: bool| {
        let t = token.get();
        if t.is_empty() || shared::auth::is_token_expired(&t) {
            go_login();
            return;
        }
        if is_initial {
            set_loading.set(true);
        }
        set_error_msg.set(String::new());
        spawn_local(async move {
            match list_media(t).await {
                Ok(res) => set_items.set(res),
                Err(e) => {
                    let err_str = e.to_string();
                    if err_str.contains("Invalid token") || err_str.contains("ExpiredSignature") {
                        #[cfg(target_arch = "wasm32")]
                        if let Some(w) = web_sys::window() {
                            if let Ok(Some(s)) = w.local_storage() {
                                let _ = s.remove_item("admin_token");
                            }
                        }
                        go_login();
                    } else {
                        set_error_msg.set(format!("Error loading media: {}", e));
                    }
                }
            }
            if is_initial {
                set_loading.set(false);
            }
        });
    };

    // Trigger initial fetch when token is loaded
    Effect::new(move || {
        fetch_media(true);
    });

    let on_upload = move |ev: ev::Event| {
        let input: HtmlInputElement = event_target(&ev);
        let files: Option<FileList> = input.files();
        if let Some(files) = files {
            if let Some(file) = files.get(0) {
                let t = token.get();
                if shared::auth::is_token_expired(&t) {
                    go_login();
                    return;
                }
                let filename = file.name();
                let file_clone = file.clone();
                set_uploading.set(true);
                set_error_msg.set(String::new());
                set_success_msg.set(String::new());

                spawn_local(async move {
                    let array_buffer_promise = file_clone.array_buffer();
                    match JsFuture::from(array_buffer_promise).await {
                        Ok(array_buffer) => {
                            let uint8_array = js_sys::Uint8Array::new(&array_buffer);
                            let bytes = uint8_array.to_vec();

                            match upload_media(t, filename.clone(), bytes).await {
                                Ok(_url) => {
                                    set_success_msg
                                        .set(format!("Successfully uploaded '{}'!", filename));
                                    fetch_media(false); // Background refresh (no blink)
                                }
                                Err(e) => {
                                    let err_str = e.to_string();
                                    if err_str.contains("Invalid token")
                                        || err_str.contains("ExpiredSignature")
                                    {
                                        #[cfg(target_arch = "wasm32")]
                                        if let Some(w) = web_sys::window() {
                                            if let Ok(Some(s)) = w.local_storage() {
                                                let _ = s.remove_item("admin_token");
                                            }
                                        }
                                        go_login();
                                    } else {
                                        set_error_msg.set(format!("Upload failed: {}", e));
                                    }
                                }
                            }
                        }
                        Err(e) => set_error_msg.set(format!("File read failed: {:?}", e)),
                    }
                    set_uploading.set(false);
                });
            }
        }
    };

    view! {
        <div class="container py-12">
            // Dropdown click outside overlay
            {move || if menu_open.get() {
                Some(view! {
                    <div
                        class="fixed inset-0 z-40 bg-transparent"
                        on:click=move |_| set_menu_open.set(false)
                    />
                })
            } else { None }}

            <div class="flex justify-between items-center mb-8">
                <div>
                    <h1 class="text-4xl mb-2">"Media Library"</h1>
                    <p class="text-muted">"Upload and manage your portfolio's public assets in GCS."</p>
                </div>
                <a href="/admin/dashboard" class="btn btn-secondary">"← Dashboard"</a>
            </div>

            // Status Messages
            {move || if !error_msg.get().is_empty() {
                Some(view! { <div class="mb-4 p-3 bg-red-50 border border-red-200 text-red-700 rounded-md text-sm">{error_msg.get()}</div> })
            } else { None }}

            {move || if !success_msg.get().is_empty() {
                Some(view! { <div class="mb-4 p-3 bg-green-50 border border-green-200 text-green-700 rounded-md text-sm">{success_msg.get()}</div> })
            } else { None }}

            // Upload Box
            <div class="card mb-8">
                <h3 class="text-xl font-bold mb-4">"Upload New Asset"</h3>
                <div class="p-8 border-2 border-dashed border-gray-300 rounded-xl text-center bg-gray-50/50 hover:border-blue-400 transition-colors">
                    <label class="cursor-pointer">
                        <div class="flex flex-col items-center gap-2">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-12 w-12 text-gray-400 mb-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
                            </svg>
                            <span class="text-blue-600 hover:text-blue-800 font-bold text-lg">
                                {move || if uploading.get() { "Uploading asset..." } else { "Choose a file to upload" }}
                            </span>
                            <span class="text-sm text-gray-500">"JPG, PNG, WebP, SVG, or MP4 up to 50MB"</span>
                        </div>
                        <input type="file" class="hidden" accept="image/*,video/*" on:change=on_upload disabled=move || uploading.get() />
                    </label>
                </div>
            </div>

            // Media Grid
            <div class="card">
                <div class="flex justify-between items-center mb-6">
                    <h3 class="text-xl font-bold">"Existing Assets"</h3>

                    <div class="flex items-center gap-2">
                        <button
                            class="btn btn-sm btn-secondary"
                            on:click=move |_| fetch_media(false)
                            disabled=move || loading.get()
                        >
                            "Refresh Grid"
                        </button>

                        // Three-dot batch commands dropdown
                        <div class="dropdown-container">
                            <button
                                class="dropdown-trigger"
                                on:click=move |_| set_menu_open.update(|open| *open = !*open)
                                title="Batch Actions"
                            >
                                <svg viewBox="0 0 24 24">
                                    <path d="M12 8c1.1 0 2-.9 2-2s-.9-2-2-2-2 .9-2 2 .9 2 2 2zm0 2c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2zm0 6c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2z"/>
                                </svg>
                            </button>

                            {move || if menu_open.get() {
                                let selected_count = selected_urls.get().len();
                                let has_selected = selected_count > 0;

                                let on_select_all = move |_| {
                                    let all_urls: HashSet<String> = items.get().iter().map(|item| item.url.clone()).collect();
                                    set_selected_urls.set(all_urls);
                                    set_menu_open.set(false);
                                };

                                let on_clear_selection = move |_| {
                                    set_selected_urls.set(HashSet::new());
                                    set_menu_open.set(false);
                                };

                                let on_copy_selected = move |_| {
                                    #[cfg(target_arch = "wasm32")]
                                    {
                                        let selected_list: Vec<String> = selected_urls.get().into_iter().collect();
                                        let joined_urls = selected_list.join("\n");
                                        let window = web_sys::window().unwrap();
                                        let navigator = window.navigator();
                                        let clipboard = navigator.clipboard();
                                        let _ = clipboard.write_text(&joined_urls);
                                        set_success_msg.set(format!("Successfully copied {} URLs to clipboard!", selected_list.len()));
                                    }
                                    set_menu_open.set(false);
                                };

                                let on_delete_selected = move |_| {
                                    set_menu_open.set(false);
                                    let selected_set = selected_urls.get();
                                    let count = selected_set.len();
                                    if count == 0 { return; }

                                    #[cfg(target_arch = "wasm32")]
                                    {
                                        let window = web_sys::window().unwrap();
                                        if !window.confirm_with_message(&format!("Are you sure you want to permanently delete {} assets?", count)).unwrap_or(false) {
                                            return;
                                        }
                                    }

                                    set_success_msg.set(String::new());
                                    set_error_msg.set(String::new());

                                    // Optimistic local update
                                    let selected_set_clone = selected_set.clone();
                                    set_items.update(move |current_items| {
                                        current_items.retain(|item| !selected_set_clone.contains(&item.url));
                                    });

                                    // Reset selection state
                                    set_selected_urls.set(HashSet::new());

                                    // Execute async deletions
                                    let t = token.get();
                                    spawn_local(async move {
                                        let mut failed = Vec::new();
                                        for url in selected_set {
                                            let path_to_remove = if let Some(stripped) =
                                                url.strip_prefix("https://storage.googleapis.com/jakewray-portfolio/")
                                            {
                                                stripped.to_string()
                                            } else if let Some(stripped) = url.strip_prefix(
                                                "https://storage.googleapis.com/download/storage/v1/b/jakewray-portfolio/o/",
                                            ) {
                                                stripped.to_string()
                                            } else {
                                                url.clone()
                                            };

                                            let decoded_path = match urlencoding::decode(&path_to_remove) {
                                                Ok(cow) => cow.into_owned(),
                                                Err(_) => path_to_remove.clone(),
                                            };

                                            match delete_media(t.clone(), decoded_path.clone()).await {
                                                Ok(_) => {}
                                                Err(e) => failed.push(format!("{}: {}", decoded_path, e)),
                                            }
                                        }

                                        if failed.is_empty() {
                                            set_success_msg.set("Permanently deleted all selected assets from GCS.".to_string());
                                        } else {
                                            set_error_msg.set(format!("Deleted other assets, but failed to delete: {}", failed.join(", ")));
                                        }
                                        fetch_media(false); // Silent refresh
                                    });
                                };

                                Some(view! {
                                    <div class="dropdown-menu">
                                        <button class="dropdown-item" on:click=on_select_all>
                                            "Select All"
                                        </button>
                                        <button class="dropdown-item" on:click=on_clear_selection disabled=move || !has_selected>
                                            "Clear Selection"
                                        </button>
                                        <div class="border-b my-1"></div>
                                        <button class="dropdown-item" on:click=on_copy_selected disabled=move || !has_selected>
                                            {move || format!("Copy Selected ({})", selected_count)}
                                        </button>
                                        <button class="dropdown-item danger" on:click=on_delete_selected disabled=move || !has_selected>
                                            {move || format!("Delete Selected ({})", selected_count)}
                                        </button>
                                    </div>
                                })
                            } else { None }}
                        </div>
                    </div>
                </div>

                {move || if loading.get() {
                    view! { <div class="py-12 text-center text-gray-400 text-lg">"Loading GCS bucket assets..."</div> }.into_any()
                } else if items.get().is_empty() {
                    view! { <div class="py-12 text-center text-gray-400 text-lg">"No assets found in your GCS bucket."</div> }.into_any()
                } else {
                    view! {
                        <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-6">
                            {items.get().into_iter().map(move |item| {
                                let url = item.url.clone();
                                let url_toggle = url.clone();
                                let url_check = url.clone();

                                let is_selected_fn = move || {
                                    selected_urls.get().contains(&url_check)
                                };

                                let toggle_fn = move || {
                                    set_selected_urls.update(|selected| {
                                        if selected.contains(&url_toggle) {
                                            selected.remove(&url_toggle);
                                        } else {
                                            selected.insert(url_toggle.clone());
                                        }
                                    });
                                };

                                view! {
                                    <AdminMediaItem
                                        item=item
                                        is_selected=is_selected_fn
                                        on_toggle=toggle_fn
                                    />
                                }
                            }).collect_view()}
                        </div>
                    }.into_any()
                }}
            </div>
        </div>
    }
}
