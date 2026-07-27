// use crate::data::journalism; // Deprecated
use crate::api::articles::{get_articles, Article};
use crate::components::media_picker::MediaPicker;
use crate::components::rich_editor::RichTextEditor;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;

fn strip_tags(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for ch in s.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ => {
                if !in_tag {
                    out.push(ch)
                }
            }
        }
    }
    out.trim().to_string()
}

fn starts_with_month(s: &str) -> bool {
    let sm = s.trim_start();
    const MONTHS: [&str; 21] = [
        "Jan.",
        "January",
        "Feb.",
        "February",
        "Mar.",
        "March",
        "Apr.",
        "April",
        "May",
        "June",
        "July",
        "Aug.",
        "August",
        "Sept.",
        "September",
        "Oct.",
        "October",
        "Nov.",
        "November",
        "Dec.",
        "December",
    ];
    MONTHS.iter().any(|m| {
        if let Some(after) = sm.strip_prefix(m) {
            // Match if it's the end of string or next char is not a letter
            after.chars().next().is_none_or(|c| !c.is_alphabetic())
        } else {
            false
        }
    })
}

fn extract_between(
    haystack: &str,
    start_pat: &str,
    end_pat: &str,
    from: usize,
) -> Option<(String, usize)> {
    let start_idx = haystack[from..].find(start_pat)? + from;
    let after = start_idx + start_pat.len();
    let end_idx = haystack[after..].find(end_pat)? + after;
    Some((
        haystack[after..end_idx].to_string(),
        end_idx + end_pat.len(),
    ))
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
            if starts_with_month(&text) {
                return Some(text);
            }
            pos = next;
        } else {
            break;
        }
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
                        let formatted_date = format_cp_style(new_date);
                        let replacement = format!(
                            "<p class=\"text-sm text-gray-500 mb-6 mt-6\">{}</p>",
                            formatted_date
                        );
                        out.replace_range(start_abs..end_abs, &replacement);
                        return out;
                    }
                }
            }
            pos = next;
        } else {
            break;
        }
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
                img_tag[after_src..]
                    .find('"')
                    .map(|src_end_rel| &img_tag[after_src..after_src + src_end_rel])
            } else {
                None
            };

            if let Some(src_url) = src {
                let is_safe_scheme = src_url.starts_with("http://")
                    || src_url.starts_with("https://")
                    || src_url.starts_with("data:image/png")
                    || src_url.starts_with("data:image/jpeg")
                    || src_url.starts_with("data:image/gif")
                    || src_url.starts_with("data:image/webp")
                    || src_url.starts_with('/');

                if is_safe_scheme {
                    let safe_url = src_url
                        .replace("&", "&amp;")
                        .replace("\"", "&quot;")
                        .replace("<", "&lt;")
                        .replace(">", "&gt;");
                    let wrapper_start = format!(
                        "<a href=\"{}\" target=\"_blank\" rel=\"noopener noreferrer\" class=\"article-image-link\">",
                        safe_url
                    );
                    let wrapper_end = "</a>";
                    let safe_img_tag =
                        format!("<img src=\"{}\" alt=\"Article Image\" />", safe_url);

                    // Replace strict range
                    let new_content = format!("{}{}{}", wrapper_start, safe_img_tag, wrapper_end);
                    out.replace_range(abs_open..abs_close, &new_content);

                    search_pos = abs_open + new_content.len();
                    continue;
                } else {
                    #[cfg(not(target_arch = "wasm32"))]
                    tracing::debug!(
                        "Skipped unsafe image scheme in journalism article: {}",
                        src_url
                    );
                    #[cfg(target_arch = "wasm32")]
                    web_sys::console::log_1(
                        &format!(
                            "Skipped unsafe image scheme in journalism article: {}",
                            src_url
                        )
                        .into(),
                    );
                }
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
            } else {
                break;
            }
        } else {
            search_pos = abs_open + 2;
        }
    }
    out
}

fn format_cp_style(date: &str) -> String {
    date.replace("January", "Jan.")
        .replace("February", "Feb.")
        .replace("August", "Aug.")
        .replace("September", "Sept.")
        .replace("October", "Oct.")
        .replace("November", "Nov.")
        .replace("December", "Dec.")
}

pub fn get_article_sort_key(article: &Article) -> String {
    #[cfg(feature = "ssr")]
    {
        if let Some(printed) = extract_printed_date(&article.content_html) {
            let (_, iso, _) = crate::api::articles::parse_article_date(&printed);
            if iso != "1970-01-01" {
                return iso;
            }
        }
        if !article.iso_date.is_empty() && article.iso_date != "1970-01-01" {
            return article.iso_date.clone();
        }
        let (_, iso, _) = crate::api::articles::parse_article_date(&article.display_date);
        iso
    }
    #[cfg(not(feature = "ssr"))]
    {
        if !article.iso_date.is_empty() && article.iso_date != "1970-01-01" {
            article.iso_date.clone()
        } else {
            article.display_date.clone()
        }
    }
}

pub fn sort_articles_newest_first(articles: &mut [Article]) {
    articles.sort_by(|a, b| {
        let key_a = get_article_sort_key(a);
        let key_b = get_article_sort_key(b);
        key_b.cmp(&key_a).then_with(|| a.title.cmp(&b.title))
    });
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
                            Ok(mut articles) => {
                                sort_articles_newest_first(&mut articles);
                                view! {
                                    <div class="journalism-grid">
                                        {articles.into_iter().map(|article| {
                                            let slug = article.slug.clone();
                                            let title = article.title.clone();
                                            let preview_text = extract_body_preview(&article.content_html)
                                                .unwrap_or_else(|| article.excerpt.clone());
                                            let image = article.images.first().cloned();
                                            let date = extract_printed_date(&article.content_html)
                                                .unwrap_or_else(|| article.display_date.clone());
                                            let date = format_cp_style(&date);

                                            view! {
                                                <A href=format!("/journalism/{}", slug) attr:class="journalism-card">
                                                    <div class="journalism-thumb">
                                                        {if let Some(ref img) = image {
                                                            view! { <img src=img.clone() class="journalism-img" alt="article thumbnail"/> }.into_any()
                                                        } else {
                                                            view! {
                                                                <svg class="journalism-img" xmlns="http://www.w3.org/2000/svg" width="400" height="300" viewBox="0 0 400 300">
                                                                    <rect width="400" height="300" fill="#e5e7eb"/>
                                                                    <text x="50%" y="50%" dominant-baseline="middle" text-anchor="middle" fill="#9ca3af" font-size="16" font-family="Inter, sans-serif">"Image coming soon"</text>
                                                                </svg>
                                                            }.into_any()
                                                        }}
                                                        // Removed duplicate placeholder div
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
                                }.into_any()
                            }
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

    use crate::api::articles::{delete_article, get_article, get_articles, save_article};

    let params = use_params_map();
    let slug = move || params.with(|p| p.get("slug").map(|s| s.to_string()).unwrap_or_default());

    let article_resource = Resource::new(slug, get_article);
    let articles_resource = Resource::new(|| (), |_| get_articles());

    // Auth State
    let (is_admin, _set_is_admin) = signal(false);
    let (token, _set_token) = signal(String::new());

    Effect::new(move || {
        #[cfg(target_arch = "wasm32")]
        {
            #[cfg(debug_assertions)]
            web_sys::console::log_1(&"Checking auth token...".into());
            if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
                if let Ok(Some(t)) = storage.get_item("admin_token") {
                    #[cfg(debug_assertions)]
                    web_sys::console::log_1(&format!("Found token: {}", t).into());
                    if !t.is_empty() {
                        if shared::auth::is_token_expired(&t) {
                            let _ = storage.remove_item("admin_token");
                            _set_is_admin.set(false);
                            _set_token.set(String::new());
                            #[cfg(debug_assertions)]
                            web_sys::console::log_1(&"Expired token cleared from storage".into());
                        } else {
                            _set_token.set(t);
                            _set_is_admin.set(true);
                            #[cfg(debug_assertions)]
                            web_sys::console::log_1(&"Admin mode enabled".into());
                        }
                    }
                } else {
                    #[cfg(debug_assertions)]
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
                        #[cfg(target_arch = "wasm32")]
                        if let Some(window) = web_sys::window() {
                            if let Ok(Some(storage)) = window.local_storage() {
                                let _ = storage.remove_item("admin_token");
                            }
                        }
                        _set_is_admin.set(false);
                        _set_token.set(String::new());
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

        let t = token.get();
        spawn_local(async move {
            match delete_article(t, slug).await {
                Ok(_) => {
                    let navigate = leptos_router::hooks::use_navigate();
                    navigate("/journalism", Default::default());
                }
                Err(e) => {
                    let err_str = e.to_string();
                    if err_str.contains("Invalid token") || err_str.contains("ExpiredSignature") {
                        #[cfg(target_arch = "wasm32")]
                        if let Some(window) = web_sys::window() {
                            if let Ok(Some(storage)) = window.local_storage() {
                                let _ = storage.remove_item("admin_token");
                            }
                        }
                        _set_is_admin.set(false);
                        _set_token.set(String::new());
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

pub fn prev_article_index(current_idx: usize, total: usize) -> Option<usize> {
    if total <= 1 {
        None
    } else {
        Some((current_idx + total - 1) % total)
    }
}

pub fn next_article_index(current_idx: usize, total: usize) -> Option<usize> {
    if total <= 1 {
        None
    } else {
        Some((current_idx + 1) % total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prev_next_article_index_cycling() {
        let total = 3;
        // Index 0 (newest article)
        assert_eq!(prev_article_index(0, total), Some(2));
        assert_eq!(next_article_index(0, total), Some(1));

        // Index 1 (middle article)
        assert_eq!(prev_article_index(1, total), Some(0));
        assert_eq!(next_article_index(1, total), Some(2));

        // Index 2 (oldest article)
        assert_eq!(prev_article_index(2, total), Some(1));
        assert_eq!(next_article_index(2, total), Some(0));
    }

    #[test]
    fn test_prev_next_article_index_single_or_empty() {
        assert_eq!(prev_article_index(0, 1), None);
        assert_eq!(next_article_index(0, 1), None);
        assert_eq!(prev_article_index(0, 0), None);
        assert_eq!(next_article_index(0, 0), None);
    }

    #[test]
    fn test_replace_date_paragraph() {
        let html = r#"<div><h4>Title</h4><p class="date">May 21, 2025</p><p>Body text</p></div>"#;
        let updated = replace_date_paragraph(html, "May 22, 2025");
        assert!(updated.contains("May 22, 2025"));
        assert!(!updated.contains("May 21, 2025"));

        let html_no_date = r#"<div><p>Just body text</p></div>"#;
        let unchanged = replace_date_paragraph(html_no_date, "May 22, 2025");
        assert_eq!(unchanged, html_no_date);
    }

    #[test]
    fn test_sort_articles_newest_first() {
        let mut articles = vec![
            Article {
                slug: "old-article".to_string(),
                title: "Old Article".to_string(),
                iso_date: "2020-07-16".to_string(),
                display_date: "July 16, 2020".to_string(),
                source_url: String::new(),
                content_html: "<p>July 16, 2020</p>".to_string(),
                images: vec![],
                captions: vec![],
                excerpt: String::new(),
                byline: None,
            },
            Article {
                slug: "mid-article".to_string(),
                title: "Mid Article".to_string(),
                iso_date: "2025-05-21".to_string(),
                display_date: "May 21, 2025".to_string(),
                source_url: String::new(),
                content_html: "<p>May 21, 2025</p>".to_string(),
                images: vec![],
                captions: vec![],
                excerpt: String::new(),
                byline: None,
            },
            Article {
                slug: "new-article".to_string(),
                title: "New Article".to_string(),
                iso_date: "2026-08-01".to_string(),
                display_date: "August 1, 2026".to_string(),
                source_url: String::new(),
                content_html: "<p>August 1, 2026</p>".to_string(),
                images: vec![],
                captions: vec![],
                excerpt: String::new(),
                byline: None,
            },
        ];

        sort_articles_newest_first(&mut articles);
        assert_eq!(articles[0].slug, "new-article");
        assert_eq!(articles[1].slug, "mid-article");
        assert_eq!(articles[2].slug, "old-article");

        // Manually update the date of old-article to be the newest
        articles[2].display_date = "October 5, 2026".to_string();
        articles[2].content_html =
            replace_date_paragraph(&articles[2].content_html, "October 5, 2026");
        articles[2].iso_date = "2026-10-05".to_string();

        sort_articles_newest_first(&mut articles);
        assert_eq!(articles[0].slug, "old-article");
        assert_eq!(articles[1].slug, "new-article");
        assert_eq!(articles[2].slug, "mid-article");
    }
}
