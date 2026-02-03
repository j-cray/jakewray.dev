// use crate::data::journalism; // Deprecated
use crate::api::articles::{get_articles, Article};
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use leptos::task::spawn_local;

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
                                            <a href=format!("/journalism/{}", slug) class="journalism-card">
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
                                            </a>
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
    let (is_admin, set_is_admin) = signal(false);
    let (token, set_token) = signal(String::new());

    Effect::new(move || {
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::console::log_1(&"Checking auth token...".into());
            if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
                if let Ok(Some(t)) = storage.get_item("admin_token") {
                     web_sys::console::log_1(&format!("Found token: {}", t).into());
                     if !t.is_empty() {
                        set_token.set(t);
                        set_is_admin.set(true);
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
    
    // Form Signals (initialized when entering edit mode or loading article)
    let (edit_title, set_edit_title) = signal(String::new());
    let (edit_date, set_edit_date) = signal(String::new()); // using iso_date or display_date?
    let (edit_byline, set_edit_byline) = signal(String::new()); // using iso_date or display_date?
    let (edit_html, set_edit_html) = signal(String::new());
    let (save_status, set_save_status) = signal(String::new());

    let turn_on_edit = move |article: &Article| {
        set_edit_title.set(article.title.clone());
        set_edit_date.set(article.display_date.clone());
        set_edit_byline.set(article.byline.clone().unwrap_or_default());
        set_edit_html.set(article.content_html.clone());
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
                                                                <div class="font-bold mb-4">{article.byline.clone().unwrap_or("By Jake Wray".to_string())}</div>
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
                                        let title = article.title.clone();
                                        let article_save = article.clone();
                                        let article_delete = article.clone();

                                        view! {
                                            <div class="edit-container p-6 bg-white border border-blue-200 rounded shadow-lg">
                                                <h2 class="text-2xl font-bold mb-4">"Editing: " {title}</h2>
                                                
                                                <div class="form-group mb-4">
                                                    <label class="block font-bold mb-1">"Headline"</label>
                                                    <input type="text" class="w-full p-2 border rounded" 
                                                        prop:value=edit_title.get()
                                                        on:input=move |ev| set_edit_title.set(event_target_value(&ev))
                                                    />
                                                </div>
                                                
                                                <div class="form-group mb-4">
                                                    <label class="block font-bold mb-1">"Display Date"</label>
                                                    <input type="text" class="w-full p-2 border rounded" 
                                                        prop:value=edit_date.get()
                                                        on:input=move |ev| set_edit_date.set(event_target_value(&ev))
                                                    />
                                                </div>

                                                <div class="form-group mb-4">
                                                    <label class="block font-bold mb-1">"Byline"</label>
                                                    <input type="text" class="w-full p-2 border rounded" 
                                                        prop:value=edit_byline.get()
                                                        on:input=move |ev| set_edit_byline.set(event_target_value(&ev))
                                                    />
                                                </div>
                                                
                                                <div class="form-group mb-4">
                                                    <label class="block font-bold mb-1">"HTML Body"</label>
                                                    <textarea class="w-full p-2 border rounded h-96 font-mono text-sm"
                                                        prop:value=edit_html.get()
                                                        on:input=move |ev| set_edit_html.set(event_target_value(&ev))
                                                    ></textarea>
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
