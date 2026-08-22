use crate::api::articles::Article;
use crate::utils::html::{extract_body_preview, extract_printed_date, format_cp_style};
use leptos::prelude::*;
use leptos_router::components::A;

pub fn render_article_card(article: &Article) -> impl IntoView {
    let slug = article.slug.clone();
    let title = article.title.clone();
    let preview_text =
        extract_body_preview(&article.content_html).unwrap_or_else(|| article.excerpt.clone());
    let image = article.images.first().cloned();
    let date =
        extract_printed_date(&article.content_html).unwrap_or_else(|| article.display_date.clone());
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
            </div>
            <div class="journalism-body">
                <p class="journalism-date">{date}</p>
                <h3 class="journalism-title">{title}</h3>
                <p class="journalism-excerpt">{preview_text}</p>
                <div class="journalism-link">"Read more →"</div>
            </div>
        </A>
    }
}
