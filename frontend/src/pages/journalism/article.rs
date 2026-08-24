use crate::api::articles::{delete_article, get_article, get_articles, save_article, Article};
use crate::components::media_picker::MediaPicker;
use crate::components::rich_editor::RichTextEditor;
use crate::context::{use_admin_context, AdminAction, AdminActionIcon};
use crate::utils::html::{
    extract_printed_date, format_cp_style, process_article_content, replace_date_paragraph,
};
use crate::utils::sorting::{next_article_index, prev_article_index};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;

#[component]
pub fn JournalismArticlePage() -> impl IntoView {
    #[cfg(target_arch = "wasm32")]
    web_sys::console::log_1(&"Rendering JournalismArticlePage".into());

    let params = use_params_map();
    let slug = move || params.with(|p| p.get("slug").map(|s| s.to_string()).unwrap_or_default());

    let article_resource = Resource::new(slug, get_article);
    let articles_resource = Resource::new(|| (), |_| get_articles());
    let admin_ctx = use_admin_context();
    let (current_article, set_current_article) = signal(None::<Article>);

    // Edit State
    let (is_editing, set_is_editing) = signal(false);

    // Form Signals
    let (edit_title, set_edit_title) = signal(String::new());
    let (edit_date, set_edit_date) = signal(String::new());
    let (edit_byline, set_edit_byline) = signal(String::new());
    let (edit_caption, set_edit_caption) = signal(String::new());
    let (edit_html, set_edit_html) = signal(String::new());
    let (edit_images, set_edit_images) = signal(Vec::<String>::new());
    let (show_media_picker, set_show_media_picker) = signal(false);
    let (save_status, set_save_status) = signal(String::new());

    let turn_on_edit = move |article: &Article| {
        set_edit_title.set(article.title.clone());
        set_edit_date.set(article.display_date.clone());
        set_edit_byline.set(article.byline.clone().unwrap_or_default());
        set_edit_caption.set(article.captions.first().cloned().unwrap_or_default());
        set_edit_html.set(article.content_html.clone());
        set_edit_images.set(article.images.clone());
        set_is_editing.set(true);
    };

    Effect::new(move || {
        if let Some(Ok(Some(a))) = article_resource.get() {
            let is_admin = admin_ctx.is_admin.get();
            crate::utils::analytics::track_article_view(&a.slug, &a.title, is_admin);
            set_current_article.set(Some(a));
        }
    });

    // Register contextual action with the persistent AdminBar
    Effect::new(move || {
        let is_adm = admin_ctx.is_admin.get();
        let is_edit = is_editing.get();
        let cur_slug = slug();
        if is_adm {
            if is_edit {
                admin_ctx.set_action(AdminAction {
                    label: "Exit Edit Mode".to_string(),
                    icon: AdminActionIcon::Close,
                    href: None,
                    on_click: Some(Callback::new(move |_| {
                        set_is_editing.set(false);
                    })),
                    is_active: true,
                });
            } else {
                admin_ctx.set_action(AdminAction {
                    label: "Edit Article".to_string(),
                    icon: AdminActionIcon::Edit,
                    href: None,
                    on_click: Some(Callback::new(move |_| {
                        if let Some(ref article) = current_article.get() {
                            turn_on_edit(article);
                        } else if let Some(Ok(Some(ref article))) = article_resource.get() {
                            turn_on_edit(article);
                        } else {
                            set_edit_title.set(cur_slug.clone());
                            set_is_editing.set(true);
                        }
                    })),
                    is_active: false,
                });
            }
        }
    });

    let on_save = move |original_article: Article| {
        let t = admin_ctx.token.get();
        spawn_local(async move {
            set_save_status.set("Saving...".to_string());
            let mut new_article = original_article.clone();
            let new_date_str = edit_date.get();
            new_article.title = edit_title.get();
            new_article.display_date = new_date_str.clone();

            new_article.byline = Some(edit_byline.get());
            new_article.captions = if edit_caption.get().trim().is_empty() {
                vec![]
            } else {
                vec![edit_caption.get()]
            };
            new_article.images = edit_images.get();
            new_article.content_html = replace_date_paragraph(&edit_html.get(), &new_date_str);

            match save_article(t, new_article).await {
                Ok(_) => {
                    set_save_status.set("Saved!".to_string());
                    set_is_editing.set(false);
                    article_resource.refetch();
                    articles_resource.refetch();
                }
                Err(e) => {
                    let err_str = e.to_string();
                    if err_str.contains("Invalid token") || err_str.contains("ExpiredSignature") {
                        admin_ctx.logout();
                        set_save_status
                            .set("Save failed: Session expired. Please log in again.".to_string());
                    } else {
                        set_save_status.set(format!("Error: {}", e));
                    }
                }
            }
        });
    };

    let on_delete = move |slug: String| {
        #[cfg(target_arch = "wasm32")]
        {
            if !web_sys::window()
                .unwrap()
                .confirm_with_message("Are you sure you want to delete this article?")
                .unwrap()
            {
                return;
            }
        }

        let t = admin_ctx.token.get();
        spawn_local(async move {
            match delete_article(t, slug).await {
                Ok(_) => {
                    let navigate = leptos_router::hooks::use_navigate();
                    navigate("/journalism", Default::default());
                }
                Err(e) => {
                    let err_str = e.to_string();
                    if err_str.contains("Invalid token") || err_str.contains("ExpiredSignature") {
                        admin_ctx.logout();
                    }
                    #[cfg(target_arch = "wasm32")]
                    let _ = web_sys::window()
                        .unwrap()
                        .alert_with_message(&format!("Error deleting: {}", e));
                    #[cfg(not(target_arch = "wasm32"))]
                    leptos::logging::error!("Error deleting: {}", e);
                }
            }
        });
    };

    view! {
        <div class="container py-12 max-w-4xl">
             <Suspense fallback=move || view! { <p>"Loading article..."</p> }>
                {move || {
                    article_resource.get().map(|res| {
                        match res {
                            Ok(Some(article)) => {
                                let display_date = extract_printed_date(&article.content_html)
                                    .unwrap_or_else(|| article.display_date.clone());
                                let display_date = format_cp_style(&display_date);
                                let title = article.title.clone();
                                let source_url = article.source_url.clone();
                                let images = article.images.clone();
                                let captions = article.captions.clone();
                                let is_terrace = source_url.contains("terracestandard.com");

                                // Render View
                                let view_mode = {
                                    let article = article.clone();
                                    move || {
                                        let article = article.clone();
                                        let content_html = process_article_content(&article.content_html);

                                        view! {
                                            <div class="article-container">
                                                <h1 class="mb-4 text-4xl font-bold text-black">{title.clone()}</h1>

                                                // Image Logic
                                                {if is_terrace || !images.is_empty() {
                                                    Some(view! {
                                                        <div class="mb-6">
                                                            {images.first().map(|url| view! {
                                                                <figure class="mb-4">
                                                                    <a href=url.clone() target="_blank" class="article-image-link">
                                                                        <img src=url.clone() class="w-full h-auto rounded-lg" alt=title.clone() />
                                                                    </a>
                                                                    {captions.first().map(|cap| view! {
                                                                        <figcaption class="mt-2 text-sm text-gray-500 italic">
                                                                            {cap.clone()}
                                                                        </figcaption>
                                                                    })}
                                                                </figure>
                                                            })}
                                                            <div class="flex flex-col text-black">
                                                                <div class="mb-4">{display_date.clone()}</div>
                                                                <div class="font-bold mb-4">
                                                                    {let b = article.byline.clone().unwrap_or_default();
                                                                     if !b.is_empty() {
                                                                         if b.to_lowercase().starts_with("by ") {
                                                                             Some(b)
                                                                         } else {
                                                                             Some(format!("By {}", b))
                                                                         }
                                                                     } else {
                                                                         None
                                                                     }}
                                                                </div>
                                                            </div>
                                                        </div>
                                                    })
                                                } else { None }}

                                                <div class="article-content prose" inner_html=content_html></div>

                                                {move || {
                                                    articles_resource.get().and_then(|res| res.ok()).and_then(|articles| {
                                                        if articles.len() <= 1 {
                                                             return None;
                                                        }
                                                        let cur_slug = slug();
                                                        let idx = articles.iter().position(|a| a.slug == cur_slug)?;
                                                        let prev_idx = prev_article_index(idx, articles.len())?;
                                                        let next_idx = next_article_index(idx, articles.len())?;
                                                        let prev_slug = articles[prev_idx].slug.clone();
                                                        let prev_title = articles[prev_idx].title.clone();
                                                        let next_slug = articles[next_idx].slug.clone();
                                                        let next_title = articles[next_idx].title.clone();

                                                        Some(view! {
                                                            <nav class="article-nav" aria-label="Article navigation">
                                                                <A href=format!("/journalism/{}", prev_slug) attr:class="article-nav-link prev">
                                                                    <span class="article-nav-label">"← Previous Article"</span>
                                                                    <span class="article-nav-title">{prev_title}</span>
                                                                </A>
                                                                <A href=format!("/journalism/{}", next_slug) attr:class="article-nav-link next">
                                                                    <span class="article-nav-label">"Next Article →"</span>
                                                                    <span class="article-nav-title">{next_title}</span>
                                                                </A>
                                                            </nav>
                                                        })
                                                    })
                                                }}
                                            </div>
                                        }.into_any()
                                    }
                                };

                                let edit_mode = {
                                    let article = article.clone();
                                    move || {
                                        let article = article.clone();
                                        let article_save = article.clone();
                                        let article_delete = article.clone();

                                        view! {
                                            <div class="edit-container w-full max-w-5xl mx-auto p-8">
                                                <div class="max-w-2xl mx-auto">
                                                    <h2 class="text-3xl font-bold mb-8 pb-4 border-b text-center">"Editing Article"</h2>

                                                    <div class="form-group mb-6">
                                                        <label class="block font-bold mb-2 text-gray-700">"Headline"</label>
                                                        <textarea class="w-full p-3 border rounded-lg text-2xl font-bold resize-none" rows="2"
                                                            prop:value=edit_title.get()
                                                            on:input=move |ev| set_edit_title.set(event_target_value(&ev))
                                                        ></textarea>
                                                    </div>

                                                    <div class="form-group mb-6">
                                                        <label class="block font-bold mb-2 text-gray-700">"Photo"</label>
                                                        <div class="flex flex-col gap-4 mb-2">
                                                            {move || {
                                                                let imgs = edit_images.get();
                                                                if let Some(src) = imgs.first() {
                                                                    view! {
                                                                        <div class="relative group w-full mt-2">
                                                                            <div class="border-2 border-gray-200 rounded-lg overflow-hidden shadow-sm">
                                                                                <img
                                                                                    src=src.clone()
                                                                                    class="w-full h-auto object-cover transition-transform duration-500 group-hover:scale-105"
                                                                                />
                                                                            </div>
                                                                            <button
                                                                                type="button"
                                                                                class="absolute -top-3 -right-3 bg-red-600 text-white rounded-full w-8 h-8 flex items-center justify-center shadow-md hover:bg-red-700 transition-colors z-10"
                                                                                on:click=move |_| set_edit_images.update(|i| { i.clear(); })
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
                                                            let current = edit_images.get().first().cloned();
                                                            Some(view! {
                                                                <div class="mt-4 border rounded p-4 bg-gray-50">
                                                                    <MediaPicker
                                                                        token=admin_ctx.token.into()
                                                                        current_image=current
                                                                        on_select=move |url| {
                                                                            set_edit_images.set(vec![url]);
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
                                                            prop:value=edit_caption.get()
                                                            on:input=move |ev| set_edit_caption.set(event_target_value(&ev))
                                                        ></textarea>
                                                    </div>

                                                    <div class="form-group mb-6">
                                                        <label class="block font-bold mb-2 text-gray-700">"Display Date"</label>
                                                        <textarea class="w-full p-3 border rounded-lg resize-none" rows="1"
                                                            prop:value=edit_date.get()
                                                            on:input=move |ev| set_edit_date.set(event_target_value(&ev))
                                                        ></textarea>
                                                    </div>

                                                    <div class="form-group mb-6">
                                                        <label class="block font-bold mb-2 text-gray-700">"Byline"</label>
                                                        <textarea class="w-full p-3 border rounded-lg resize-none font-bold" rows="1"
                                                            prop:value=edit_byline.get()
                                                            on:input=move |ev| set_edit_byline.set(event_target_value(&ev))
                                                        ></textarea>
                                                    </div>

                                                    <div class="form-group mb-6">
                                                        <label class="block font-bold mb-2 text-gray-700">"Article Text"</label>
                                                        <RichTextEditor
                                                            value=edit_html
                                                            on_change=move |new_val| set_edit_html.set(new_val)
                                                        />
                                                    </div>

                                                    <div class="flex gap-4 items-center">
                                                        <button class="btn btn-primary" on:click=move |_| on_save(article_save.clone())>
                                                            "Save Changes"
                                                        </button>
                                                        <button class="btn btn-secondary" on:click=move |_| set_is_editing.set(false)>
                                                            "Cancel"
                                                        </button>
                                                        <div class="flex-grow"></div>
                                                        <button class="btn btn-danger bg-red-600 text-white hover:bg-red-700" on:click=move |_| on_delete(article_delete.slug.clone())>
                                                            "Delete Article"
                                                        </button>
                                                    </div>
                                                    <p class="mt-2 text-sm text-gray-600">{save_status.get()}</p>
                                                </div>
                                            </div>
                                        }.into_any()
                                    }
                                };

                                view! {
                                    <div>
                                    {move || if is_editing.get() { edit_mode() } else { view_mode() }}
                                    </div>
                                }.into_any()

                            },
                            Ok(None) => view! { <div><p>"Article not found."</p></div> }.into_any(),
                            Err(e) => view! { <p class="text-red-500">"Error loading article: " {e.to_string()}</p> }.into_any(),
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}
