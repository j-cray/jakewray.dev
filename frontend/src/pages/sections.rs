use crate::data::journalism;
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

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
    MONTHS.iter().any(|m| sm.starts_with(m))
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

#[component]
pub fn JournalismPage() -> impl IntoView {
    let articles = journalism::all_articles();

    view! {
        <div class="container py-12">
            <h1 class="text-4xl mb-4">"Journalism"</h1>
            <p class="text-gray-700 mb-10 max-w-3xl">
                "Reporting on northern communities, Indigenous culture, and public interest stories."
            </p>

            <div class="journalism-grid">
                {articles
                    .iter()
                    .map(|article| {
                        let slug = article.slug.clone();
                        let title = article.title.clone();
                        let preview_text = extract_body_preview(&article.content_html)
                            .unwrap_or_else(|| article.excerpt.clone());
                        let image = article.images.get(0).cloned();
                        let thumb_src = image.clone().unwrap_or_else(|| "data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='400' height='300' viewBox='0 0 400 300'><rect width='400' height='300' fill='%23e5e7eb'/><text x='50%' y='50%' dominant-baseline='middle' text-anchor='middle' fill='%239ca3af' font-size='16' font-family='Inter, sans-serif'>Image coming soon</text></svg>".to_string());
                        let date = extract_printed_date(&article.content_html)
                            .unwrap_or_else(|| article.display_date.clone());
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
                    })
                    .collect_view()}
            </div>
        </div>
    }
}

#[component]
pub fn JournalismArticlePage() -> impl IntoView {
    let params = use_params_map();
    let slug = move || params.with(|p| p.get("slug").map(|s| s.to_string()).unwrap_or_default());
    let article = move || journalism::find_article(&slug());

    view! {
        <div class="container py-12 max-w-4xl">
            {move || {
                article()
                    .map(|article| {
                        let display_date = extract_printed_date(&article.content_html)
                            .unwrap_or_else(|| article.display_date.clone());
                        let title = article.title.clone();
                        let source_url = article.source_url.clone();
                        let images = article.images.clone();
                        let content_html = article.content_html.clone();
                        // remove first subhead h4 from article content
                        let content_html = {
                            if let Some((_, end)) = extract_between(&content_html, "<h4", "</h4>", 0) {
                                let start = content_html.find("<h4").unwrap_or(0);
                                let mut s = content_html.clone();
                                s.replace_range(start..end, "");
                                s
                            } else { content_html }
                        };
                        view! {
                            <div>
                                <p class="text-sm text-gray-500 mb-2">{display_date}</p>
                                <h1 class="mb-4 text-4xl font-bold text-gray-900">{title}</h1>



                                <div class="article-content prose" inner_html=content_html></div>
                            </div>
                        }
                        .into_any()
                    })
                    .unwrap_or_else(|| view! { <div><p>"Article not found."</p></div> }.into_any())
            }}
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
             <h1 class="text-4xl mb-6">"Programming"</h1>
             <p class="text-muted">"GitHub Showcase."</p>
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
