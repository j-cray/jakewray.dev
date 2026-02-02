// Force rebuild to include updated JSON
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JournalismArticle {
    pub slug: String,
    pub title: String,
    pub iso_date: String,
    pub display_date: String,
    pub source_url: String,
    pub content_html: String,
    pub images: Vec<String>,
    #[serde(default)]
    pub captions: Vec<String>,
    pub excerpt: String,
    #[serde(default)]
    pub byline: Option<String>,
}

static ARTICLES: Lazy<Vec<JournalismArticle>> = Lazy::new(|| {
    let mut parsed: Vec<JournalismArticle> = serde_json::from_str(include_str!("journalism.json"))
        .expect("journalism.json should parse");

    parsed.sort_by(|a, b| b.iso_date.cmp(&a.iso_date));
    leptos::logging::log!("Loaded {} articles from JSON", parsed.len());
    parsed
});

pub fn all_articles() -> &'static [JournalismArticle] {
    &ARTICLES
}

pub fn find_article(slug: &str) -> Option<JournalismArticle> {
    ARTICLES.iter().find(|article| article.slug == slug).cloned()
}
