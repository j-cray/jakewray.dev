use crate::api::articles::{list_media, upload_media, delete_media, MediaItem};
use leptos::ev;
use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen_futures::JsFuture;
use web_sys::{FileList, HtmlInputElement};

#[component]
pub fn AdminMedia() -> impl IntoView {
    let (items, set_items) = signal(Vec::::new());
    let (loading, set_loading) = signal(true);
    let (uploading, set_uploading) = signal(false);
    let (error_msg, set_error_msg) = signal(String::new());
    let (success_msg, set_success_msg) = signal(String::new());
    let (token, set_token) = signal(String::new());

    #[cfg(target_arch = "wasm32")]
    let navigate = leptos_router::hooks::use_navigate();

    // Check auth on load
    Effect::new(move || {
        #[cfg(target_arch = "wasm32")]
        {
            let window = web_sys::window().unwrap();
            let local_storage = window.local_storage().unwrap().unwrap();
            if let Ok(Some(t)) = local_storage.get_item("admin_token") {
                if !t.is_empty() {
                    set_token.set(t);
                    return;
                }
            }
            navigate("/admin/login", Default::default());
        }
    });

    let fetch_media = move |is_initial: bool| {
        let t = token.get();
        if t.is_empty() {
            return;
        }
        if is_initial {
            set_loading.set(true);
        }
        set_error_msg.set(String::new());
        spawn_local(async move {
            match list_media(t).await {
                Ok(res) => set_items.set(res),
                Err(e) => set_error_msg.set(format!("Error loading media: {}", e)),
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
        let files: Option = input.files();
        if let Some(files) = files {
            if let Some(file) = files.get(0) {
                let t = token.get();
                let f_clone = fetch_media;
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
                                    set_success_msg.set(format!("Successfully uploaded '{}'!", filename));
                                    f_clone(false); // Background refresh (no blink)
                                }
                                Err(e) => set_error_msg.set(format!("Upload failed: {}", e)),
                            }
                        }
                        Err(e) => set_error_msg.set(format!("File read failed: {:?}", e)),
                    }
                    set_uploading.set(false);
                });
            }
        }
    };

    let on_delete = move |url: String| {
        #[cfg(target_arch = "wasm32")]
        {
            let window = web_sys::window().unwrap();
            if !window
                .confirm_with_message(&format!("Are you sure you want to permanently delete this file?"))
                .unwrap_or(false)
            {
                return;
            }
        }

        let t = token.get();
        let f_clone = fetch_media;
        set_error_msg.set(String::new());
        set_success_msg.set(String::new());

        // Optimistic Update: instantly remove the deleted item from UI list in-memory!
        // This makes the UI feel buttery-smooth and instantaneous.
        let url_to_remove = url.clone();
        set_items.update(move |current_items| {
            current_items.retain(|item| item.url != url_to_remove);
        });

        spawn_local(async move {
            // Extract the GCS object name from the URL
            let path_to_remove = if let Some(stripped) = url.strip_prefix("https://storage.googleapis.com/jakewray-portfolio/") {
                stripped.to_string()
            } else if let Some(stripped) = url.strip_prefix("https://storage.googleapis.com/download/storage/v1/b/jakewray-portfolio/o/") {
                stripped.to_string()
            } else {
                url.clone()
            };

            // URL decode the path name with explicit matching to avoid type inference issues
            let decoded_path = match urlencoding::decode(&path_to_remove) {
                Ok(cow) => cow.into_owned(),
                Err(_) => path_to_remove.clone(),
            };

            match delete_media(t, decoded_path.clone()).await {
                Ok(_) => {
                    set_success_msg.set(format!("Permanently deleted asset from GCS."));
                    f_clone(false); // Silent background sync refresh
                }
                Err(e) => {
                    set_error_msg.set(format!("Delete failed: {}", e));
                    f_clone(true); // Bring the deleted item back on error
                }
            }
        });
    };

    let copy_to_clipboard = move |url: String| {
        #[cfg(target_arch = "wasm32")]
        {
            let window = web_sys::window().unwrap();
            let navigator = window.navigator();
            let clipboard = navigator.clipboard();
            let _ = clipboard.write_text(&url);
            set_success_msg.set(format!("URL copied to clipboard!"));
        }
    };

    view! {
        <div class="container py-12">
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

                        <div class="flex flex-col items-center gap-2">

                            <span class="text-blue-600 hover:text-blue-800 font-bold text-lg">
                                {move || if uploading.get() { "Uploading asset..." } else { "Choose a file to upload" }}
                            </span>
                            <span class="text-sm text-gray-500">"JPG, PNG, WebP, SVG, or MP4 up to 50MB"</span>
                        </div>


                </div>
            </div>

            // Media Grid
            <div class="card">
                <div class="flex justify-between items-center mb-6">
                    <h3 class="text-xl font-bold">"Existing Assets"</h3>

                </div>

                {move || if loading.get() {
                    view! { <div class="py-12 text-center text-gray-400 text-lg">"Loading GCS bucket assets..."</div> }.into_any()
                } else if items.get().is_empty() {
                    view! { <div class="py-12 text-center text-gray-400 text-lg">"No assets found in your GCS bucket."</div> }.into_any()
                } else {
                    let on_del = on_delete.clone();
                    let on_copy = copy_to_clipboard.clone();

                    view! {
                        <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-6">
                            {items.get().into_iter().map(move |item| {
                                let url = item.url.clone();
                                let u_del = url.clone();
                                let u_copy = url.clone();
                                let od = on_del.clone();
                                let oc = on_copy.clone();

                                view! {
                                    <div class="relative group border border-gray-200 rounded-xl overflow-hidden bg-gray-50 shadow-sm hover:shadow-md hover:border-blue-300 transition-all flex flex-col">
                                        // Image thumbnail
                                        <div class="aspect-square bg-white relative overflow-hidden flex items-center justify-center">
                                            <img src="url.clone()" alt="item.name.clone()" class="w-full h-full object-cover">
                                        </div>

                                        // Asset Details
                                        <div class="p-3 flex flex-col gap-2 flex-grow">
                                            <span class="text-sm font-semibold truncate text-gray-700" title="item.name.clone()">{item.name.clone()}</span>

                                            // Actions
                                            <div class="grid grid-cols-2 gap-2 mt-auto">


                                            </div>
                                        </div>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    }.into_any()
                }}
            </div>
        </div>
    }
}
