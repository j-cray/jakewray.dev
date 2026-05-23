use crate::api::articles::{list_media, upload_media, delete_media, MediaItem};
use leptos::ev;
use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen_futures::JsFuture;
use web_sys::{FileList, HtmlInputElement};

#[component]
pub fn AdminMedia() -> impl IntoView {
    let (items, set_items) = signal(Vec::<MediaItem>::new());
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

    let fetch_media = move || {
        let t = token.get();
        if t.is_empty() {
            return;
        }
        set_loading.set(true);
        set_error_msg.set(String::new());
        spawn_local(async move {
            match list_media(t).await {
                Ok(res) => set_items.set(res),
                Err(e) => set_error_msg.set(format!("Error loading media: {}", e)),
            }
            set_loading.set(false);
        });
    };

    // Trigger fetch when token is loaded
    Effect::new(move || {
        fetch_media();
    });

    let on_upload = move |ev: ev::Event| {
        let input: HtmlInputElement = event_target(&ev);
        let files: Option<FileList> = input.files();
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
                                    f_clone(); // Refresh grid
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
        set_loading.set(true);
        set_error_msg.set(String::new());
        set_success_msg.set(String::new());

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
                    f_clone();
                }
                Err(e) => {
                    set_error_msg.set(format!("Delete failed: {}", e));
                    set_loading.set(false);
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
                        <input type="file" class="hidden" accept="image/*,video/*" on:change=on_upload disabled=uploading />
                    </label>
                </div>
            </div>

            // Media Grid
            <div class="card">
                <div class="flex justify-between items-center mb-6">
                    <h3 class="text-xl font-bold">"Existing Assets"</h3>
                    <button class="btn btn-sm btn-secondary" on:click=move |_| fetch_media() disabled=loading>"Refresh Grid"</button>
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
                                            <img src=url.clone() alt=item.name.clone() class="w-full h-full object-cover" />
                                        </div>

                                        // Asset Details
                                        <div class="p-3 flex flex-col gap-2 flex-grow">
                                            <span class="text-sm font-semibold truncate text-gray-700" title=item.name.clone()>{item.name}</span>
                                            
                                            // Actions
                                            <div class="grid grid-cols-2 gap-2 mt-auto">
                                                <button 
                                                    class="btn btn-secondary text-xs py-1.5 px-2 justify-center" 
                                                    on:click=move |_| oc(u_copy.clone())
                                                    title="Copy asset GCS URL"
                                                >
                                                    "Copy URL"
                                                </button>
                                                <button 
                                                    class="btn btn-danger bg-red-600 text-white hover:bg-red-700 text-xs py-1.5 px-2 justify-center" 
                                                    on:click=move |_| od(u_del.clone())
                                                    title="Permanently delete asset"
                                                >
                                                    "Delete"
                                                </button>
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
