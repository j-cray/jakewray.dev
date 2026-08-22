pub mod schedule_modal;
pub mod types;

pub use schedule_modal::ScheduleModal;
pub use types::{
    current_date_string, current_iso_datetime_local, get_current_time_string, ComposerDraftData,
};

use crate::api::articles::{get_article, save_article, Article};
use crate::components::media_picker::MediaPicker;
use crate::components::rich_editor::RichTextEditor;
use crate::utils::slug::sanitize_slug;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::{use_location, use_navigate};
use std::sync::Arc;

#[component]
pub fn AdminComposer() -> impl IntoView {
    let navigate = use_navigate();
    let location = use_location();
    let (token, set_token) = signal(String::new());
    let _ = &set_token;

    let (title, set_title) = signal(String::new());
    let (slug, set_slug) = signal(String::new());
    let (images, set_images) = signal(Vec::<String>::new());
    let (caption, set_caption) = signal(String::new());
    let (display_date, set_display_date) = signal(current_date_string());
    let (byline, set_byline) = signal("By Jake Wray".to_string());
    let (content, set_content) = signal("<p>Start writing post...</p>".to_string());
    let (_post_status, set_post_status) = signal("draft".to_string());

    let (show_media_picker, set_show_media_picker) = signal(false);
    let (save_status, set_save_status) = signal(String::new());
    let (autosave_status, set_autosave_status) = signal(String::new());
    let (is_saving, set_is_saving) = signal(false);

    // Schedule post modal states
    let (show_schedule_modal, set_show_schedule_modal) = signal(false);
    let (scheduled_datetime, set_scheduled_datetime) = signal(current_iso_datetime_local());

    // Flag to prevent overwriting during initial load
    let (is_loaded, set_is_loaded) = signal(false);
    let _ = &is_loaded;

    let _nav_auth = navigate.clone();
    Effect::new(move || {
        #[cfg(target_arch = "wasm32")]
        {
            if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
                if let Ok(Some(t)) = storage.get_item("admin_token") {
                    if !t.is_empty() && !shared::auth::is_token_expired(&t) {
                        set_token.set(t);
                    } else {
                        _nav_auth("/admin/login", Default::default());
                    }
                } else {
                    _nav_auth("/admin/login", Default::default());
                }
            }
        }
    });

    // Check for query slug or load local draft on mount
    Effect::new(move || {
        let query_slug = location.query.get().get("slug");
        if let Some(s) = query_slug {
            if !s.is_empty() {
                spawn_local(async move {
                    if let Ok(Some(article)) = get_article(s).await {
                        set_title.set(article.title);
                        set_slug.set(article.slug);
                        set_images.set(article.images);
                        if let Some(cap) = article.captions.first() {
                            set_caption.set(cap.clone());
                        }
                        set_display_date.set(article.display_date);
                        if let Some(b) = article.byline {
                            set_byline.set(b);
                        }
                        set_content.set(article.content_html);
                        if let Some(st) = article.status {
                            set_post_status.set(st);
                        }
                        set_is_loaded.set(true);
                    }
                });
                return;
            }
        }

        // If no slug param, check for locally autosaved draft
        #[cfg(target_arch = "wasm32")]
        {
            if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
                if let Ok(Some(draft_json)) = storage.get_item("composer_draft_data") {
                    if let Ok(draft) = serde_json::from_str::<ComposerDraftData>(&draft_json) {
                        if !draft.title.is_empty()
                            || !draft.content.is_empty()
                            || !draft.slug.is_empty()
                        {
                            set_title.set(draft.title);
                            set_slug.set(draft.slug);
                            set_images.set(draft.images);
                            set_caption.set(draft.caption);
                            if !draft.display_date.is_empty() {
                                set_display_date.set(draft.display_date);
                            }
                            if !draft.byline.is_empty() {
                                set_byline.set(draft.byline);
                            }
                            if !draft.content.is_empty() {
                                set_content.set(draft.content);
                            }
                            set_autosave_status.set(format!("Draft restored from local cache"));
                        }
                    }
                }
            }
        }
        set_is_loaded.set(true);
    });

    // Autosave to localStorage on changes after initial load
    Effect::new(move || {
        #[cfg(target_arch = "wasm32")]
        if is_loaded.get() {
            let draft = ComposerDraftData {
                title: title.get(),
                slug: slug.get(),
                images: images.get(),
                caption: caption.get(),
                display_date: display_date.get(),
                byline: byline.get(),
                content: content.get(),
                updated_at: current_iso_datetime_local(),
            };

            if let Ok(json_str) = serde_json::to_string(&draft) {
                if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
                    let _ = storage.set_item("composer_draft_data", &json_str);
                    set_autosave_status.set("Autosaved locally".to_string());
                }
            }
        }
    });

    let clear_local_draft = move || {
        #[cfg(target_arch = "wasm32")]
        {
            if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
                let _ = storage.remove_item("composer_draft_data");
            }
        }
        set_title.set(String::new());
        set_slug.set(String::new());
        set_images.set(Vec::new());
        set_caption.set(String::new());
        set_display_date.set(current_date_string());
        set_byline.set("By Jake Wray".to_string());
        set_content.set("<p>Start writing post...</p>".to_string());
        set_autosave_status.set("Local draft cleared".to_string());
    };

    let save_post_with_status = Arc::new(
        move |target_status: &'static str, scheduled_iso: Option<String>| {
            let t = token.get();
            if shared::auth::is_token_expired(&t) {
                #[cfg(target_arch = "wasm32")]
                if let Some(window) = web_sys::window() {
                    if let Ok(Some(storage)) = window.local_storage() {
                        let _ = storage.remove_item("admin_token");
                    }
                }
                set_save_status.set("Session expired. Please log in again.".to_string());
                return;
            }

            let post_title = title.get();
            if post_title.trim().is_empty() {
                set_save_status.set("Please enter a headline before saving.".to_string());
                return;
            }

            let mut final_slug = slug.get();
            if final_slug.trim().is_empty() {
                final_slug = sanitize_slug(&post_title);
            } else {
                final_slug = sanitize_slug(&final_slug);
            }

            set_is_saving.set(true);
            set_save_status.set(format!("Saving as {}...", target_status));

            let nav = navigate.clone();
            let final_display_date = display_date.get();
            let date_for_db = if let Some(iso) = scheduled_iso {
                iso
            } else {
                final_display_date.clone()
            };

            let new_article = Article {
                slug: final_slug,
                title: post_title,
                iso_date: String::new(),
                display_date: date_for_db,
                source_url: String::new(),
                content_html: content.get(),
                images: images.get(),
                captions: if caption.get().trim().is_empty() {
                    vec![]
                } else {
                    vec![caption.get()]
                },
                excerpt: String::new(),
                byline: if byline.get().trim().is_empty() {
                    None
                } else {
                    Some(byline.get())
                },
                status: Some(target_status.to_string()),
            };

            spawn_local(async move {
                match save_article(t, new_article).await {
                    Ok(_) => {
                        // Clear local storage draft after successful save/publish/schedule
                        #[cfg(target_arch = "wasm32")]
                        {
                            if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
                                let _ = storage.remove_item("composer_draft_data");
                            }
                        }

                        if target_status == "published" {
                            set_save_status
                                .set("Published successfully! Redirecting...".to_string());
                            nav("/blog", Default::default());
                        } else if target_status == "scheduled" {
                            set_save_status
                                .set("Scheduled successfully! Redirecting...".to_string());
                            nav("/admin/dashboard", Default::default());
                        } else {
                            set_save_status.set("Draft saved to database.".to_string());
                            set_post_status.set("draft".to_string());
                            set_is_saving.set(false);
                        }
                    }
                    Err(e) => {
                        set_save_status.set(format!("Error saving post: {}", e));
                        set_is_saving.set(false);
                    }
                }
            });
        },
    );

    let save_pub = save_post_with_status.clone();
    let save_draft = save_post_with_status.clone();
    let save_sched = save_post_with_status.clone();

    view! {
        <div class="container py-12">
            <div class="edit-container w-full max-w-5xl mx-auto p-8">
                <div class="max-w-2xl mx-auto">
                    <div class="flex items-center justify-between mb-8 pb-4 border-b">
                        <h2 class="text-3xl font-bold">"Post Composer"</h2>
                        <div class="flex items-center gap-3">
                            {move || {
                                let st = autosave_status.get();
                                if !st.is_empty() {
                                    view! {
                                        <span class="text-xs text-gray-500 bg-gray-100 px-3 py-1.5 rounded-full flex items-center gap-1.5 font-medium">
                                            <span class="w-2 h-2 rounded-full bg-emerald-500 animate-pulse"></span>
                                            {st}
                                        </span>
                                    }.into_any()
                                } else {
                                    view! { <span class="hidden" /> }.into_any()
                                }
                            }}
                            <button
                                type="button"
                                class="text-xs text-red-600 hover:text-red-800 font-semibold px-2 py-1"
                                on:click=move |_| clear_local_draft()
                                title="Reset form and clear local draft"
                            >
                                "Clear Draft"
                            </button>
                        </div>
                    </div>

                    <div class="form-group mb-6">
                        <label class="block font-bold mb-2 text-gray-700">"Headline"</label>
                        <textarea class="w-full p-3 border rounded-lg text-2xl font-bold resize-none" rows="2"
                            prop:value=title.get()
                            on:input=move |ev| set_title.set(event_target_value(&ev))
                            placeholder="Enter post headline..."
                        ></textarea>
                    </div>

                    <div class="form-group mb-6">
                        <label class="block font-bold mb-2 text-gray-700">"Slug (Optional)"</label>
                        <input type="text" class="w-full p-3 border rounded-lg text-sm text-gray-600"
                            prop:value=slug.get()
                            on:input=move |ev| set_slug.set(event_target_value(&ev))
                            placeholder="Auto-generated from title if blank..."
                        />
                    </div>

                    <div class="form-group mb-6">
                        <label class="block font-bold mb-2 text-gray-700">"Photo"</label>
                        <div class="flex flex-col gap-4 mb-2">
                            {move || {
                                let imgs = images.get();
                                if let Some(src) = imgs.first() {
                                    view! {
                                        <div class="relative group w-full mt-2">
                                            <div class="border-2 border-gray-200 rounded-lg overflow-hidden shadow-sm">
                                                <img
                                                    src=src.clone()
                                                    class="w-full h-auto object-cover transition-transform duration-500 group-hover:scale-105"
                                                    alt="Post image preview"
                                                />
                                            </div>
                                            <button
                                                type="button"
                                                class="absolute -top-3 -right-3 bg-red-600 text-white rounded-full w-8 h-8 flex items-center justify-center shadow-md hover:bg-red-700 transition-colors z-10"
                                                on:click=move |_| set_images.update(|i| { i.clear(); })
                                                title="Remove Image"
                                            >
                                                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                                                    <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
                                                </svg>
                                            </button>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! { <span class="hidden" /> }.into_any()
                                }
                            }}
                            <button
                                type="button"
                                class="btn btn-sm btn-secondary w-auto self-start flex items-center gap-2"
                                on:click=move |_| set_show_media_picker.set(!show_media_picker.get())
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
                                </svg>
                                {move || if show_media_picker.get() { "Close Picker" } else { "Add Image" }}
                            </button>
                        </div>

                        {move || if show_media_picker.get() {
                            let current = images.get().first().cloned();
                            Some(view! {
                                <div class="mt-4 border rounded p-4 bg-gray-50">
                                    <MediaPicker
                                        token=token.into()
                                        current_image=current
                                        on_select=move |url| {
                                            set_images.set(vec![url]);
                                            set_show_media_picker.set(false);
                                        }
                                    />
                                </div>
                            })
                        } else { None }}
                    </div>

                    <div class="form-group mb-6">
                        <label class="block font-bold mb-2 text-gray-700">"Caption"</label>
                        <textarea class="w-full p-3 border rounded-lg resize-y" rows="2"
                            prop:value=caption.get()
                            on:input=move |ev| set_caption.set(event_target_value(&ev))
                            placeholder="Photo caption (optional)..."
                        ></textarea>
                    </div>

                    <div class="form-group mb-6">
                        <label class="block font-bold mb-2 text-gray-700">"Display Date"</label>
                        <textarea class="w-full p-3 border rounded-lg resize-none" rows="1"
                            prop:value=display_date.get()
                            on:input=move |ev| set_display_date.set(event_target_value(&ev))
                        ></textarea>
                    </div>

                    <div class="form-group mb-6">
                        <label class="block font-bold mb-2 text-gray-700">"Byline"</label>
                        <textarea class="w-full p-3 border rounded-lg resize-none font-bold" rows="1"
                            prop:value=byline.get()
                            on:input=move |ev| set_byline.set(event_target_value(&ev))
                        ></textarea>
                    </div>

                    <div class="form-group mb-6">
                        <label class="block font-bold mb-2 text-gray-700">"Article Text"</label>
                        <RichTextEditor
                            value=content
                            on_change=move |new_val| set_content.set(new_val)
                        />
                    </div>

                    <ScheduleModal
                        show=show_schedule_modal.into()
                        on_close=Callback::new(move |_| set_show_schedule_modal.set(false))
                        scheduled_datetime=scheduled_datetime.into()
                        set_scheduled_datetime=set_scheduled_datetime
                        set_save_status=set_save_status
                        on_confirm=Callback::new(move |(status, dt): (&'static str, Option<String>)| {
                            save_sched(status, dt);
                        })
                    />

                    <div class="flex flex-wrap gap-3 items-center mt-8 pt-6 border-t">
                        <button
                            type="button"
                            class="btn btn-primary flex items-center gap-2"
                            on:click=move |_| save_pub("published", None)
                            disabled=move || is_saving.get()
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8" />
                            </svg>
                            "Publish Post"
                        </button>

                        <button
                            type="button"
                            class="btn btn-secondary flex items-center gap-2"
                            on:click=move |_| set_show_schedule_modal.set(true)
                            disabled=move || is_saving.get()
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-sky-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                            </svg>
                            "Schedule Post"
                        </button>

                        <button
                            type="button"
                            class="btn btn-secondary flex items-center gap-2"
                            on:click=move |_| save_draft("draft", None)
                            disabled=move || is_saving.get()
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-amber-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-3 3m0 0l-3-3m3 3V4" />
                            </svg>
                            "Save Draft"
                        </button>

                        <a href="/admin/dashboard" class="btn btn-secondary text-gray-500 hover:text-gray-700">
                            "Cancel"
                        </a>

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
            </div>
        </div>
    }
}
