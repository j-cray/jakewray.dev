use crate::api::media::{list_media, upload_media, MediaItem};
use leptos::ev;
use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen_futures::JsFuture;
use web_sys::{FileList, HtmlInputElement};

#[component]
pub fn MediaPicker<F>(
    token: Signal<String>,
    on_select: F,
    current_image: Option<String>,
) -> impl IntoView
where
    F: Fn(String) + 'static + Send + Sync + Clone,
{
    let (items, set_items) = signal(Vec::<MediaItem>::new());
    let (loading, set_loading) = signal(true);
    let (uploading, set_uploading) = signal(false);
    let (error_msg, set_error_msg) = signal(String::new());

    let fetch_media = move || {
        set_loading.set(true);
        let t = token.get();
        if shared::auth::is_token_expired(&t) {
            #[cfg(target_arch = "wasm32")]
            if let Some(window) = web_sys::window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    let _ = storage.remove_item("admin_token");
                }
            }
            set_error_msg.set(
                "Session expired (Expired or missing token). Please log in again to upload media."
                    .to_string(),
            );
            set_loading.set(false);
            return;
        }

        spawn_local(async move {
            match list_media(t).await {
                Ok(res) => set_items.set(res),
                Err(e) => {
                    let err_str = e.to_string();
                    if err_str.contains("Invalid token") || err_str.contains("ExpiredSignature") {
                        #[cfg(target_arch = "wasm32")]
                        if let Some(window) = web_sys::window() {
                            if let Ok(Some(storage)) = window.local_storage() {
                                let _ = storage.remove_item("admin_token");
                            }
                        }
                        set_error_msg.set(
                            "Session expired. Please log in again to manage media.".to_string(),
                        );
                    } else {
                        set_error_msg.set(format!("Error: {}", e));
                    }
                }
            }
            set_loading.set(false);
        });
    };

    // Initial fetch
    Effect::new(move || {
        fetch_media();
    });

    let on_upload = move |ev: ev::Event| {
        let input: HtmlInputElement = event_target(&ev);
        let files: Option<FileList> = input.files();
        if let Some(files) = files {
            if let Some(file) = files.get(0) {
                let t = token.get();
                if shared::auth::is_token_expired(&t) {
                    #[cfg(target_arch = "wasm32")]
                    if let Some(window) = web_sys::window() {
                        if let Ok(Some(storage)) = window.local_storage() {
                            let _ = storage.remove_item("admin_token");
                        }
                    }
                    set_error_msg
                        .set("Upload failed: Session expired. Please log in again.".to_string());
                    return;
                }
                let f_clone = fetch_media;
                let filename = file.name();
                let file_clone = file.clone(); // web_sys::File is Clone (JsValue wrapper)
                set_uploading.set(true);

                spawn_local(async move {
                    // Read file as bytes via web_sys
                    let array_buffer_promise = file_clone.array_buffer();
                    match JsFuture::from(array_buffer_promise).await {
                        Ok(array_buffer) => {
                            let uint8_array = js_sys::Uint8Array::new(&array_buffer);
                            let bytes = uint8_array.to_vec();

                            match upload_media(t, filename, bytes).await {
                                Ok(_url) => {
                                    f_clone(); // Refresh list
                                }
                                Err(e) => {
                                    let err_str = e.to_string();
                                    if err_str.contains("Invalid token")
                                        || err_str.contains("ExpiredSignature")
                                    {
                                        #[cfg(target_arch = "wasm32")]
                                        if let Some(window) = web_sys::window() {
                                            if let Ok(Some(storage)) = window.local_storage() {
                                                let _ = storage.remove_item("admin_token");
                                            }
                                        }
                                        set_error_msg.set(
                                            "Upload failed: Session expired. Please log in again."
                                                .to_string(),
                                        );
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
        <div class="media-picker bg-gray-50 border rounded-lg p-4">
            <div class="flex justify-between items-center mb-4">
                <h3 class="text-lg font-bold">"Media Library"</h3>
                <button class="btn btn-sm btn-secondary" on:click=move |_| fetch_media()>"Refresh"</button>
            </div>

            {move || if !error_msg.get().is_empty() {
                let msg = error_msg.get();
                let is_expired = msg.contains("Session expired") || msg.contains("Invalid token");
                Some(view! {
                    <div class="p-3 mb-3 bg-red-50 border border-red-200 rounded-lg">
                        <p class="text-red-600 font-medium text-sm mb-1">{msg}</p>
                        {if is_expired {
                            Some(view! {
                                <a href="/admin/login" class="inline-block mt-1 text-xs text-blue-600 hover:text-blue-800 font-bold underline">
                                    "Log in again ->"
                                </a>
                            })
                        } else {
                            None
                        }}
                    </div>
                })
            } else { None }}


            <div class="mb-6 p-6 border-2 border-dashed border-gray-300 rounded-xl text-center bg-white hover:border-blue-400 transition-colors">
                <label class="cursor-pointer">
                    <div class="flex flex-col items-center gap-1.5">
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-8 w-8 text-gray-400 mb-1" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
                        </svg>
                        <span class="text-blue-600 hover:text-blue-800 font-bold text-sm">
                            {move || if uploading.get() { "Uploading asset..." } else { "Click to upload file" }}
                        </span>
                        <span class="text-xs text-gray-500">"JPG, PNG, WebP, SVG, or MP4 up to 50MB"</span>
                    </div>
                    <input type="file" class="hidden" accept="image/*,video/*" on:change=on_upload disabled=uploading />
                </label>
            </div>

            <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-4 max-h-96 overflow-y-auto p-2">
                {move || if loading.get() {
                    view! { <div class="col-span-full py-8 text-center text-gray-400">"Loading media..."</div> }.into_any()
                } else if items.get().is_empty() {
                    view! { <div class="col-span-full py-8 text-center text-gray-400">"No images found."</div> }.into_any()
                } else {
                    let on_select = on_select.clone();
                    let current_img = current_image.clone();

                    items.get().into_iter().map(move |item| {
                        let url = item.url.clone();
                        let is_selected = current_img.as_ref() == Some(&url);
                        let os = on_select.clone();
                        let u = url.clone();

                        view! {
                            <div
                                class=move || format!(
                                    "relative aspect-square border-2 rounded-lg overflow-hidden cursor-pointer hover:border-blue-400 transition-colors {}",
                                    if is_selected { "border-blue-600 ring-2 ring-blue-200" } else { "border-transparent" }
                                )
                                on:click=move |_| os(u.clone())
                            >
                                <img src=url.clone() alt=item.name.clone() class="w-full h-full object-cover" />
                                {if is_selected {
                                    Some(view! {
                                        <div class="absolute top-1 right-1 bg-blue-600 text-white rounded-full p-1">
                                            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                                                <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                                            </svg>
                                        </div>
                                    })
                                } else { None }}
                            </div>
                        }
                    }).collect_view().into_any()
                }}
            </div>
        </div>
    }
}
