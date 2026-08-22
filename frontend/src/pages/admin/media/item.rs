use crate::api::media::MediaItem;
use leptos::ev;
use leptos::prelude::*;

#[component]
pub fn AdminMediaItem(
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
