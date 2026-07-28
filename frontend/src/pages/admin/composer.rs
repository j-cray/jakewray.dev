use crate::api::articles::{save_article, Article};
use crate::components::media_picker::MediaPicker;
use crate::components::rich_editor::RichTextEditor;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;

fn current_date_string() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        let date = js_sys::Date::new_0();
        let months = [
            "January",
            "February",
            "March",
            "April",
            "May",
            "June",
            "July",
            "August",
            "September",
            "October",
            "November",
            "December",
        ];
        let month = months[date.get_month() as usize % 12];
        let day = date.get_date();
        let year = date.get_full_year();
        format!("{} {}, {}", month, day, year)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        "July 27, 2026".to_string()
    }
}

#[component]
pub fn AdminComposer() -> impl IntoView {
    let navigate = use_navigate();
    let (token, _set_token) = signal(String::new());

    let (title, set_title) = signal(String::new());
    let (slug, set_slug) = signal(String::new());
    let (images, set_images) = signal(Vec::<String>::new());
    let (caption, set_caption) = signal(String::new());
    let (display_date, set_display_date) = signal(current_date_string());
    let (byline, set_byline) = signal("By Jake Wray".to_string());
    let (content, set_content) = signal("<p>Start writing post...</p>".to_string());

    let (show_media_picker, set_show_media_picker) = signal(false);
    let (save_status, set_save_status) = signal(String::new());
    let (is_saving, set_is_saving) = signal(false);

    let _nav_auth = navigate.clone();
    Effect::new(move || {
        #[cfg(target_arch = "wasm32")]
        {
            if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
                if let Ok(Some(t)) = storage.get_item("admin_token") {
                    if !t.is_empty() && !shared::auth::is_token_expired(&t) {
                        _set_token.set(t);
                    } else {
                        _nav_auth("/admin/login", Default::default());
                    }
                } else {
                    _nav_auth("/admin/login", Default::default());
                }
            }
        }
    });

    let nav_pub = navigate;
    let on_publish = move || {
        let t = token.get();
        let title_val = title.get();
        if title_val.trim().is_empty() {
            set_save_status.set("Please enter a title for the post.".to_string());
            return;
        }

        set_is_saving.set(true);
        set_save_status.set("Publishing post...".to_string());

        let images_vec = images.get();
        let caption_val = caption.get();
        let captions_vec = if caption_val.trim().is_empty() {
            vec![]
        } else {
            vec![caption_val]
        };

        let generated_slug = title_val
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-");

        let article_slug = if slug.get().trim().is_empty() {
            generated_slug
        } else {
            slug.get().trim().to_string()
        };

        let nav = nav_pub.clone();
        let new_article = Article {
            slug: article_slug,
            title: title_val,
            iso_date: String::new(),
            display_date: display_date.get(),
            source_url: String::new(),
            content_html: content.get(),
            images: images_vec,
            captions: captions_vec,
            excerpt: String::new(),
            byline: if byline.get().trim().is_empty() {
                None
            } else {
                Some(byline.get())
            },
        };

        spawn_local(async move {
            match save_article(t, new_article).await {
                Ok(_) => {
                    set_save_status.set("Published successfully! Redirecting...".to_string());
                    nav("/blog", Default::default());
                }
                Err(e) => {
                    set_save_status.set(format!("Error saving post: {}", e));
                    set_is_saving.set(false);
                }
            }
        });
    };

    view! {
        <div class="container py-12">
            <div class="edit-container w-full max-w-5xl mx-auto p-8">
                <div class="max-w-2xl mx-auto">
                    <h2 class="text-3xl font-bold mb-8 pb-4 border-b text-center">"Compose New Post"</h2>

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

                    <div class="flex gap-4 items-center mt-8">
                        <button class="btn btn-primary" on:click=move |_| on_publish() disabled=move || is_saving.get()>
                            "Publish Post"
                        </button>
                        <a href="/blog" class="btn btn-secondary">
                            "Cancel"
                        </a>
                        <div class="flex-grow"></div>
                        {move || {
                            let st = save_status.get();
                            if !st.is_empty() {
                                view! { <span class="text-sm font-semibold text-gray-600">{st}</span> }.into_any()
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
