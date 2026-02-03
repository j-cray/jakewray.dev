use leptos::prelude::*;
use crate::api::articles::{list_media, upload_media, MediaItem};
use leptos::task::spawn_local;
use leptos::ev;
use web_sys::HtmlInputElement;
use wasm_bindgen_futures::JsFuture;

#[component]
pub fn MediaPicker<F>(
    token: Signal<String>,
    on_select: F,
    current_image: Option<String>
) -> impl IntoView 
where F: Fn(String) + 'static + Send + Sync + Clone
{
    let (items, set_items) = signal(Vec::<MediaItem>::new());
    let (loading, set_loading) = signal(true);
    let (uploading, set_uploading) = signal(false);
    let (error_msg, set_error_msg) = signal(String::new());

    let fetch_media = {
        let token = token.clone();
        move || {
            set_loading.set(true);
            let t = token.get();
            spawn_local(async move {
                match list_media(t).await {
                    Ok(res) => set_items.set(res),
                    Err(e) => set_error_msg.set(format!("Error: {}", e)),
                }
                set_loading.set(false);
            });
        }
    };

    // Initial fetch
    Effect::new({
        let fetch = fetch_media.clone();
        move || { fetch(); }
    });

    let on_upload = {
        let token = token.clone();
        let fetch = fetch_media.clone();
        move |ev: ev::Event| {
            let input: HtmlInputElement = event_target(&ev);
            let files = input.files();
            if let Some(files) = files {
                if let Some(file) = files.get(0) {
                    let t = token.get();
                    let f_clone = fetch.clone();
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
                                    },
                                    Err(e) => set_error_msg.set(format!("Upload failed: {}", e)),
                                }
                            },
                            Err(e) => set_error_msg.set(format!("File read failed: {:?}", e)),
                        }
                        set_uploading.set(false);
                    });
                }
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
                Some(view! { <p class="text-red-500 mb-2">{error_msg.get()}</p> })
            } else { None }}

            <div class="mb-6 p-4 border-2 border-dashed border-gray-300 rounded text-center">
                <label class="cursor-pointer">
                    <span class="text-blue-600 hover:text-blue-800 font-medium">
                        {move || if uploading.get() { "Uploading..." } else { "Click to upload new image" }}
                    </span>
                    <input type="file" class="hidden" accept="image/*" on:change=on_upload disabled=uploading />
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
