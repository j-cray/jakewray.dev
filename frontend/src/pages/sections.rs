// use crate::data::journalism; // Deprecated
use crate::api::articles::{get_articles, Article};
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use leptos::task::spawn_local;
use leptos_router::components::A;
use crate::components::media_picker::MediaPicker;
use leptos::wasm_bindgen::JsCast;

fn strip_tags(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for ch in s.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ => if !in_tag { out.push(ch) },
        }
    }
    out.trim().to_string()
}

fn starts_with_month(s: &str) -> bool {
    let sm = s.trim_start();
    const MONTHS: [&str; 21] = [
        "Jan.", "January", "Feb.", "February", "Mar.", "March", "Apr.", "April",
        "May", "June", "July", "Aug.", "August", "Sept.", "September", "Oct.",
        "October", "Nov.", "November", "Dec.", "December",
    ];
    MONTHS.iter().any(|m| {
        if sm.starts_with(m) {
            let after = &sm[m.len()..];
            // Match if it's the end of string or next char is not a letter
            after.chars().next().map_or(true, |c| !c.is_alphabetic())
        } else {
            false
        }
    })
}

fn extract_between(haystack: &str, start_pat: &str, end_pat: &str, from: usize) -> Option<(String, usize)> {
    let start_idx = haystack[from..].find(start_pat)? + from;
    let after = start_idx + start_pat.len();
    let end_idx = haystack[after..].find(end_pat)? + after;
    Some((haystack[after..end_idx].to_string(), end_idx + end_pat.len()))
}

#[allow(dead_code)]
fn extract_subhead(html: &str) -> Option<String> {
    let (inner, _) = extract_between(html, "<h4", "</h4>", 0)?;
    // drop attributes in opening tag
    let open_end = inner.find('>')? + 1;
    Some(strip_tags(&inner[open_end..]))
}

fn extract_printed_date(html: &str) -> Option<String> {
    // Prefer the first <p> after the first </h4>, else the first <p>
    let after_h4 = html.find("</h4>").map(|idx| idx + 5).unwrap_or(0);
    let mut pos = after_h4;
    for _ in 0..5 {
        if let Some((p_inner, next)) = extract_between(html, "<p", "</p>", pos) {
            let open_end = p_inner.find('>').map(|i| i + 1).unwrap_or(0);
            let text = strip_tags(&p_inner[open_end..]);
            if starts_with_month(&text) { return Some(text); }
            pos = next;
        } else { break; }
    }
    None
}

fn extract_body_preview(html: &str) -> Option<String> {
    // Find paragraphs after the h4; skip date/byline; use the first body paragraph
    let after_h4 = html.find("</h4>").map(|idx| idx + 5).unwrap_or(0);
    let mut pos = after_h4;
    for _ in 0..12 {
        let (p_inner, next) = extract_between(html, "<p", "</p>", pos)?;
        let open_end = p_inner.find('>').map(|i| i + 1).unwrap_or(0);
        let text = strip_tags(&p_inner[open_end..]);
        let t = text.trim();
        if !t.is_empty() && !starts_with_month(t) && !t.starts_with("By ") {
            return Some(t.to_string());
        }
        pos = next;
    }
    None
}

#[allow(dead_code)]
fn replace_date_paragraph(html: &str, new_date: &str) -> String {
    // Reuse extract logic to find the range, then replace it
     let after_h4 = html.find("</h4>").map(|idx| idx + 5).unwrap_or(0);
    let mut pos = after_h4;
    for _ in 0..5 {
        if let Some((p_inner, next)) = extract_between(html, "<p", "</p>", pos) {
            let open_end = p_inner.find('>').map(|i| i + 1).unwrap_or(0);
            let text = strip_tags(&p_inner[open_end..]);
            if starts_with_month(&text) {
                if let Some(start_rel) = html[pos..].find("<p") {
                    let start_abs = pos + start_rel;
                     let after_start = start_abs + 2; // <p len
                    if let Some(end_rel) = html[after_start..].find("</p>") {
                         let end_abs = after_start + end_rel + 4; // </p> len
                          let mut out = html.to_string();
                          // Construct replacement paragraph
                          let replacement = format!("<p class=\"text-sm text-gray-500 mb-6 mt-6\">{}</p>", new_date);
                          out.replace_range(start_abs..end_abs, &replacement);
                          return out;
                    }
                }
            }
            pos = next;
        } else { break; }
    }
    html.to_string()
}

fn bold_byline(html: &str) -> String {
    let mut out = html.to_string();
    let mut search_pos = 0;

    // Loop to find <p...>By ...</p>
    // We iterate manually to handle string mutation
    while let Some(open_rel) = out[search_pos..].find("<p") {
        let abs_open = search_pos + open_rel;

        // Find end of opening tag >
        if let Some(close_bracket_rel) = out[abs_open..].find('>') {
            let abs_content_start = abs_open + close_bracket_rel + 1;

            // Find closing </p>
            if let Some(close_p_rel) = out[abs_content_start..].find("</p>") {
                let abs_content_end = abs_content_start + close_p_rel;
                let content = &out[abs_content_start..abs_content_end];

                // Check if content starts with "By "
                // We use trim() to ignore leading whitespace/newlines
                if content.trim().starts_with("By ") && content.len() < 100 {
                    // Inject <strong> wrapping the content
                    // Note: This replaces the inner content with <strong>...</strong>
                    let new_content = format!("<strong>{}</strong>", content);
                    out.replace_range(abs_content_start..abs_content_end, &new_content);

                    // Update search_pos to skip past this paragraph
                    search_pos = abs_content_start + new_content.len() + 4; // +4 for </p>
                    continue;
                }

                search_pos = abs_content_end + 4;
            } else {
                // Malformed HTML, just break or skip
                break;
            }
        } else {
            // Malformed opening tag
            search_pos = abs_open + 2;
        }
    }
    out
}

fn linkify_images(html: &str) -> String {
    // Find <img ... src="..." ...> and wrap in <a href="..." target="_blank" class="article-image-link">...</a>
    let mut out = html.to_string();
    let mut search_pos = 0;

    while let Some(open_rel) = out[search_pos..].find("<img") {
        let abs_open = search_pos + open_rel;

        // find end of tag
        if let Some(close_rel) = out[abs_open..].find('>') {
            let abs_close = abs_open + close_rel + 1;
            let img_tag = &out[abs_open..abs_close];

            // Extract src
            let src = if let Some(src_start_rel) = img_tag.find("src=\"") {
                let after_src = src_start_rel + 5;
                if let Some(src_end_rel) = img_tag[after_src..].find('"') {
                    Some(&img_tag[after_src..after_src + src_end_rel])
                } else { None }
            } else { None };

            if let Some(src_url) = src {
                 let wrapper_start = format!("<a href=\"{}\" target=\"_blank\" class=\"article-image-link\">", src_url);
                 let wrapper_end = "</a>";

                 // Replace strict range
                 let new_content = format!("{}{}{}", wrapper_start, img_tag, wrapper_end);
                 out.replace_range(abs_open..abs_close, &new_content);

                 search_pos = abs_open + new_content.len();
                 continue;
            }
             search_pos = abs_close;

        } else {
             search_pos = abs_open + 4;
        }
    }
    out
}

fn italicize_origin_line(html: &str) -> String {
    let mut out = html.to_string();
    let mut search_pos = 0;

    while let Some(open_rel) = out[search_pos..].find("<p") {
        let abs_open = search_pos + open_rel;

        if let Some(close_bracket_rel) = out[abs_open..].find('>') {
            let abs_content_start = abs_open + close_bracket_rel + 1;

            if let Some(close_p_rel) = out[abs_content_start..].find("</p>") {
                let abs_content_end = abs_content_start + close_p_rel;
                let content = &out[abs_content_start..abs_content_end];

                // Case-insensitive check for the specific phrase
                if content.to_lowercase().contains("originally appeared in") {
                    let new_content = format!("<em>{}</em>", content);
                    out.replace_range(abs_content_start..abs_content_end, &new_content);

                    search_pos = abs_content_start + new_content.len() + 4;
                    continue;
                }

                search_pos = abs_content_end + 4;
            } else { break; }
        } else { search_pos = abs_open + 2; }
    }
    out
}

fn format_cp_style(date: &str) -> String {
    let date = date.replace("January", "Jan.")
        .replace("February", "Feb.")
        .replace("August", "Aug.")
        .replace("September", "Sept.")
        .replace("October", "Oct.")
        .replace("November", "Nov.")
        .replace("December", "Dec.");
    date
}

#[component]
pub fn JournalismPage() -> impl IntoView {
    let articles_resource = Resource::new(|| (), |_| get_articles());

    view! {
        <div class="container py-12">
            <h1 class="text-4xl mb-4">"Journalism"</h1>
            <p class="text-gray-700 mb-10 max-w-3xl">
                "Reporting on northern communities, Indigenous culture, and public interest stories."
            </p>

            <Suspense fallback=move || view! { <p>"Loading articles..."</p> }>
                {move || {
                    articles_resource.get().map(|res| {
                        match res {
                            Ok(articles) => view! {
                                <div class="journalism-grid">
                                    {articles.into_iter().map(|article| {
                                        let slug = article.slug.clone();
                                        let title = article.title.clone();
                                        let preview_text = extract_body_preview(&article.content_html)
                                            .unwrap_or_else(|| article.excerpt.clone());
                                        let image = article.images.get(0).cloned();
                                        let thumb_src = image.clone().unwrap_or_else(|| "data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='400' height='300' viewBox='0 0 400 300'><rect width='400' height='300' fill='%23e5e7eb'/><text x='50%' y='50%' dominant-baseline='middle' text-anchor='middle' fill='%239ca3af' font-size='16' font-family='Inter, sans-serif'>Image coming soon</text></svg>".to_string());
                                        let date = extract_printed_date(&article.content_html)
                                            .unwrap_or_else(|| article.display_date.clone());
                                        let date = format_cp_style(&date);

                                        view! {
                                            <A href=format!("/journalism/{}", slug) attr:class="journalism-card">
                                                <div class="journalism-thumb">
                                                    <img src=thumb_src class="journalism-img" alt="article thumbnail"/>
                                                    {image.is_none().then(|| view! { <div class="journalism-placeholder-text">"Image coming soon"</div> })}
                                                </div>
                                                <div class="journalism-body">
                                                    <p class="journalism-date">{date}</p>
                                                    <h3 class="journalism-title">{title}</h3>
                                                    <p class="journalism-excerpt">{preview_text}</p>
                                                    <div class="journalism-link">"Read more →"</div>
                                                </div>
                                            </A>
                                        }
                                    }).collect_view()}
                                </div>
                            }.into_any(),
                            Err(e) => view! { <p class="text-red-500">"Error loading articles: " {e.to_string()}</p> }.into_any()
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}

#[component]
pub fn JournalismArticlePage() -> impl IntoView {
    #[cfg(target_arch = "wasm32")]
    web_sys::console::log_1(&"Rendering JournalismArticlePage".into());

    use crate::api::articles::{get_article, save_article, delete_article};

    let params = use_params_map();
    let slug = move || params.with(|p| p.get("slug").map(|s| s.to_string()).unwrap_or_default());

    let article_resource = Resource::new(slug, |s| get_article(s));

    // Auth State
    let (is_admin, _set_is_admin) = signal(false);
    let (token, _set_token) = signal(String::new());

    Effect::new(move || {
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::console::log_1(&"Checking auth token...".into());
            if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
                if let Ok(Some(t)) = storage.get_item("admin_token") {
                     web_sys::console::log_1(&format!("Found token: {}", t).into());
                     if !t.is_empty() {
                        _set_token.set(t);
                        _set_is_admin.set(true);
                        web_sys::console::log_1(&"Admin mode enabled".into());
                     }
                } else {
                    web_sys::console::log_1(&"No token found in localStorage".into());
                }
            }
        }
    });

    // Edit State
    let (is_editing, set_is_editing) = signal(false);

    // Form Signals
    let (edit_title, set_edit_title) = signal(String::new());
    let (edit_date, set_edit_date) = signal(String::new());
    let (edit_byline, set_edit_byline) = signal(String::new());
    let (edit_caption, set_edit_caption) = signal(String::new()); // New caption signal
    let (edit_html, set_edit_html) = signal(String::new());
    let (edit_images, set_edit_images) = signal(Vec::<String>::new());
    let (show_media_picker, set_show_media_picker) = signal(false);
    let (show_html_code, set_show_html_code) = signal(false); // Toggle for RTE
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

    let on_save = move |original_article: Article| {
        let t = token.get();
        spawn_local(async move {
            set_save_status.set("Saving...".to_string());
            let mut new_article = original_article.clone();
            new_article.title = edit_title.get();
            new_article.display_date = edit_date.get();
            new_article.byline = Some(edit_byline.get());
            new_article.captions = if edit_caption.get().trim().is_empty() { vec![] } else { vec![edit_caption.get()] };
            new_article.images = edit_images.get();
            new_article.content_html = edit_html.get();

            match save_article(t, new_article).await {
                Ok(_) => {
                    set_save_status.set("Saved!".to_string());
                    set_is_editing.set(false);
                    article_resource.refetch();
                },
                Err(e) => set_save_status.set(format!("Error: {}", e)),
            }
        });
    };

    let on_delete = move |slug: String| {
        #[cfg(target_arch = "wasm32")]
        {
            if !web_sys::window().unwrap().confirm_with_message("Are you sure you want to delete this article?").unwrap() {
                return;
            }
        }

        let t = token.get();
        spawn_local(async move {
            match delete_article(t, slug).await {
                Ok(_) => {
                    let navigate = leptos_router::hooks::use_navigate();
                    navigate("/journalism", Default::default());
                },
                Err(e) => {
                    #[cfg(target_arch = "wasm32")]
                    let _ = web_sys::window().unwrap().alert_with_message(&format!("Error deleting: {}", e));
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
                                let is_terrace = source_url.contains("terracestandard.com"); // Check logic

                                // Render View
                                let view_mode = {
                                    let article = article.clone(); // Clone for capture
                                    move || {
                                        let article = article.clone(); // Clone for execution
                                        // Transformations for view logic (can move to a helper)
                                        let content_html = {
                                             let mut s = article.content_html.clone();
                                             if let Some(start) = s.find("<h4") {
                                                 if let Some(end) = s[start..].find("</h4>") {
                                                     s.replace_range(start..start + end + 5, "");
                                                 }
                                             }
                                             let s = italicize_origin_line(&s);
                                             let s = bold_byline(&s);
                                             linkify_images(&s)
                                        };

                                        view! {
                                            <div class="article-container">
                                                {
                                                    let admin_article = article.clone(); // Capture in outer closure environment
                                                    move || {
                                                        // Clone for this execution to prevent moving `admin_article` out of environment
                                                        let a = admin_article.clone();
                                                        is_admin.get().then(move || {
                                                            view! {
                                                                <div class="mb-4 p-4 bg-gray-100 border rounded flex gap-2">
                                                                    <span class="font-bold text-gray-500">"Admin Mode"</span>
                                                                    <button class="btn btn-sm btn-primary" on:click=move |_| turn_on_edit(&a)>"Edit Article"</button>
                                                                </div>
                                                            }
                                                        })
                                                    }
                                                }

                                                <h1 class="mb-4 text-4xl font-bold text-gray-900">{title.clone()}</h1>

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
                                                            <div class="flex flex-col text-gray-900">
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
                                            </div>
                                        }.into_any()
                                    }
                                };

                                let edit_mode = {
                                    let article = article.clone();
                                    move || {
                                        let article = article.clone();
                                        // let title = article.title.clone(); // Removed unused
                                        let article_save = article.clone();
                                        let article_delete = article.clone();

                                        view! {
                                            <div class="edit-container w-full max-w-5xl mx-auto p-8 bg-white border border-blue-200 rounded-xl shadow-2xl">
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
                                                                        token=token.into()
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
                                                        <div class="flex justify-between items-end mb-2">
                                                            <label class="block font-bold mb-0 text-gray-700">"Article Text"</label>

                                                            <div class="flex gap-2 items-center">
                                                                // Toolbar
                                                                <div class="flex bg-gray-100 rounded-lg border overflow-hidden mr-4">
                                                                    <button type="button" class="p-2 hover:bg-gray-200 text-gray-700 font-bold" title="Bold"
                                                                        on:mousedown=move |ev| { ev.prevent_default(); }
                                                                        on:click=move |_| {
                                                                            if let Ok(doc) = web_sys::window().unwrap().document().unwrap().dyn_into::<web_sys::HtmlDocument>() {
                                                                                let _ = doc.exec_command("bold");
                                                                            }
                                                                        }
                                                                    >
                                                                        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                                                                            <path d="M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.38-8.379-2.83-2.828z" />
                                                                            // Actually let's use a "B" icon or generic
                                                                            <path fill-rule="evenodd" d="M6 4a1 1 0 011-1h4a3 3 0 011.69 5.483 3 3 0 01-1.258.468A3 3 0 0113 14h-6a1 1 0 01-1-1V4zm2 2v2h3a1 1 0 100-2H8zm0 4v2h4a1 1 0 100-2H8z" clip-rule="evenodd" />
                                                                        </svg>
                                                                    </button>
                                                                    <button type="button" class="p-2 hover:bg-gray-200 text-gray-700 italic font-serif" title="Italic"
                                                                        on:mousedown=move |ev| { ev.prevent_default(); }
                                                                        on:click=move |_| {
                                                                            if let Ok(doc) = web_sys::window().unwrap().document().unwrap().dyn_into::<web_sys::HtmlDocument>() {
                                                                                let _ = doc.exec_command("italic");
                                                                            }
                                                                        }
                                                                    >
                                                                        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                                                                            <path fill-rule="evenodd" d="M6 4a1 1 0 011-1h.22a1 1 0 01.993.883l3.5 13.5a1 1 0 01-1.926.66l-.667-2.543H6.77l-1.332 2.664A1 1 0 014.544 18H4a1 1 0 01-1-1v-2a1 1 0 011-1h1.11l1.89-3.78L6 4z" clip-rule="evenodd" />
                                                                             // This is Font... let's just use text "I"
                                                                             <text x="6" y="15" font-family="serif" font-style="italic" font-weight="bold" font-size="14">"I"</text>
                                                                        </svg>
                                                                    </button>
                                                                    <button type="button" class="p-2 hover:bg-gray-200 text-gray-700" title="Link"
                                                                        on:mousedown=move |ev| { ev.prevent_default(); }
                                                                        on:click=move |_| {
                                                                            if let Ok(Some(url)) = web_sys::window().unwrap().prompt_with_message("Enter URL:") {
                                                                                if let Ok(doc) = web_sys::window().unwrap().document().unwrap().dyn_into::<web_sys::HtmlDocument>() {
                                                                                    let _ = doc.exec_command_with_show_ui_and_value("createLink", false, &url);
                                                                                }
                                                                            }
                                                                        }
                                                                    >
                                                                        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                                                                            <path fill-rule="evenodd" d="M12.586 4.586a2 2 0 112.828 2.828l-3 3a2 2 0 01-2.828 0 1 1 0 00-1.414 1.414 4 4 0 005.656 0l3-3a4 4 0 00-5.656-5.656l-1.5 1.5a1 1 0 101.414 1.414l1.5-1.5zm-5 5a2 2 0 012.828 0 1 1 0 101.414-1.414 4 4 0 00-5.656 0l-3 3a4 4 0 105.656 5.656l1.5-1.5a1 1 0 10-1.414-1.414l-1.5 1.5a2 2 0 11-2.828-2.828l3-3z" clip-rule="evenodd" />
                                                                        </svg>
                                                                    </button>
                                                                </div>

                                                                // View Toggles
                                                                <div class="flex bg-gray-100 rounded-lg border overflow-hidden">
                                                                    <button
                                                                        class=move || format!("p-2 {}", if !show_html_code.get() { "bg-white shadow-sm text-blue-600" } else { "text-gray-500 hover:bg-gray-50" })
                                                                        on:click=move |_| set_show_html_code.set(false)
                                                                        title="Visual Preview"
                                                                    >
                                                                        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                                                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                                                                        </svg>
                                                                    </button>
                                                                    <button
                                                                        class=move || format!("p-2 {}", if show_html_code.get() { "bg-white shadow-sm text-blue-600" } else { "text-gray-500 hover:bg-gray-50" })
                                                                        on:click=move |_| set_show_html_code.set(true)
                                                                        title="HTML Code"
                                                                    >
                                                                        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4" />
                                                                        </svg>
                                                                    </button>
                                                                </div>
                                                            </div>
                                                        </div>

                                                        {move || if show_html_code.get() {
                                                            view! {
                                                                <textarea class="w-full p-4 border rounded-lg h-[600px] font-mono text-sm bg-gray-50 text-gray-900"
                                                                    prop:value=edit_html.get()
                                                                    on:input=move |ev| set_edit_html.set(event_target_value(&ev))
                                                                ></textarea>
                                                            }.into_any()
                                                        } else {
                                                            view! {
                                                                <div
                                                                    class="w-full p-6 border rounded-lg h-[600px] overflow-y-auto prose max-w-none bg-white text-black focus:ring-2 focus:ring-blue-500 focus:outline-none"
                                                                    contenteditable="true"
                                                                    inner_html=edit_html.get_untracked()
                                                                    on:input=move |ev| {
                                                                        set_edit_html.set(event_target::<web_sys::HtmlElement>(&ev).inner_html());
                                                                    }
                                                                ></div>
                                                            }.into_any()
                                                        }}
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

#[component]
pub fn PersonalPage() -> impl IntoView {
    view! {
        <div class="container py-12">
            <h1 class="text-4xl mb-6">"Personal"</h1>
            <p class="text-gray-600 mb-8">"Blog, Creative Writing, Photography, and Videography."</p>

            <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                <a href="/personal/blog" class="card hover:shadow-lg transition-shadow">
                    <h3 class="text-xl font-bold mb-2">"Blog"</h3>
                    <p class="text-muted">"Personal thoughts and musings"</p>
                </a>

                <a href="/personal/writing" class="card hover:shadow-lg transition-shadow">
                    <h3 class="text-xl font-bold mb-2">"Creative Writing"</h3>
                    <p class="text-muted">"Stories, novels, and poetry"</p>
                </a>

                <div class="card opacity-50">
                    <h3 class="text-xl font-bold mb-2">"Photography"</h3>
                    <p class="text-muted">"Coming soon"</p>
                </div>

                <div class="card opacity-50">
                    <h3 class="text-xl font-bold mb-2">"Videography"</h3>
                    <p class="text-muted">"Coming soon"</p>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn CreativeWritingPage() -> impl IntoView {
    view! {
        <div class="container py-12">
            <h1 class="text-4xl mb-6">"Creative Writing"</h1>
             <p class="text-muted">"Stories, Novels, and Poetry."</p>
        </div>
    }
}

#[component]
pub fn MusicPage() -> impl IntoView {
    view! {
        <div class="container py-12">
            <h1 class="text-4xl mb-6">"Music"</h1>
             <p class="text-muted">"Original compositions."</p>
        </div>
    }
}

#[component]
pub fn VisualArtPage() -> impl IntoView {
    view! {
        <div class="container py-12">
            <h1 class="text-4xl mb-6">"Visual Art"</h1>
             <p class="text-muted">"Drawings and Digital Art."</p>
        </div>
    }
}

#[component]
pub fn ProgrammingPage() -> impl IntoView {
    view! {
        <div class="container py-12">
             <h1 class="text-4xl mb-6">"Code"</h1>
             <p class="text-muted">"GitHub Showcase. Coming soon..."</p>
        </div>
    }
}

#[component]
pub fn PersonalBlogPage() -> impl IntoView {
    view! {
        <div class="container py-12">
            <h1 class="text-4xl mb-6">"Blog"</h1>
            <p class="text-muted">"Personal thoughts and musings."</p>
        </div>
    }
}
