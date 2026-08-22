use crate::api::articles::Article;
#[cfg(feature = "ssr")]
use crate::utils::html::extract_printed_date;
#[cfg(feature = "ssr")]
use crate::utils::slug::parse_article_date;

pub const HIGHLIGHT_SLUGS: &[&str] = &[
    "ksi-lisims-bc-hydros-north-coast-transmission-line-to-be-fast-tracked-pm",
    "candidates-spar-over-healthcare-at-burns-lake-all-candidates-forum",
    "foster-family-loses-home-to-fire",
    "skeena-voices-the-culture-and-the-art-saved-my-life",
];

pub fn get_article_sort_key(article: &Article) -> String {
    #[cfg(feature = "ssr")]
    {
        if let Some(printed) = extract_printed_date(&article.content_html) {
            let (_, iso, _) = parse_article_date(&printed);
            if iso != "1970-01-01" {
                return iso;
            }
        }
        if !article.iso_date.is_empty() && article.iso_date != "1970-01-01" {
            return article.iso_date.clone();
        }
        let (_, iso, _) = parse_article_date(&article.display_date);
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

pub fn extract_highlight_articles(articles: &[Article], highlight_slugs: &[&str]) -> Vec<Article> {
    highlight_slugs
        .iter()
        .filter_map(|slug| articles.iter().find(|a| a.slug == *slug).cloned())
        .collect()
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
    use crate::utils::html::replace_date_paragraph;

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
                status: None,
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
                status: None,
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
                status: None,
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

    #[test]
    fn test_extract_highlight_articles() {
        let articles = vec![
            Article {
                slug: "article-1".to_string(),
                title: "Article 1".to_string(),
                iso_date: "2025-01-01".to_string(),
                display_date: "Jan 1, 2025".to_string(),
                source_url: String::new(),
                content_html: String::new(),
                images: vec![],
                captions: vec![],
                excerpt: String::new(),
                byline: None,
                status: None,
            },
            Article {
                slug: "article-2".to_string(),
                title: "Article 2".to_string(),
                iso_date: "2025-01-02".to_string(),
                display_date: "Jan 2, 2025".to_string(),
                source_url: String::new(),
                content_html: String::new(),
                images: vec![],
                captions: vec![],
                excerpt: String::new(),
                byline: None,
                status: None,
            },
            Article {
                slug: "article-3".to_string(),
                title: "Article 3".to_string(),
                iso_date: "2025-01-03".to_string(),
                display_date: "Jan 3, 2025".to_string(),
                source_url: String::new(),
                content_html: String::new(),
                images: vec![],
                captions: vec![],
                excerpt: String::new(),
                byline: None,
                status: None,
            },
        ];

        let highlights =
            extract_highlight_articles(&articles, &["article-3", "article-1", "non-existent-slug"]);
        assert_eq!(highlights.len(), 2);
        assert_eq!(highlights[0].slug, "article-3");
        assert_eq!(highlights[1].slug, "article-1");
    }
}
