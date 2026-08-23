use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BlogPostItem {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub excerpt: String,
    pub display_date: String,
    pub iso_date: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[server(GetRecentBlogPosts, "/api")]
pub async fn get_recent_blog_posts(
    limit: Option<usize>,
) -> Result<Vec<BlogPostItem>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use sqlx::SqlitePool;
        let pool = use_context::<SqlitePool>()
            .ok_or_else(|| ServerFnError::new("SqlitePool not found in Leptos context"))?;

        let limit_val = limit.unwrap_or(6).min(50) as i64;

        let rows = sqlx::query(
            "SELECT id, slug, title, content, published_at, tags FROM blog_posts ORDER BY published_at DESC LIMIT ?",
        )
        .bind(limit_val)
        .fetch_all(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Database query failed: {}", e)))?;

        let mut posts = Vec::new();
        for row in rows {
            use sqlx::Row;
            let id: String = row.get("id");
            let slug: String = row.get("slug");
            let title: String = row.get("title");
            let content: String = row.get("content");
            let published_at: String = row.get("published_at");
            let tags_str: Option<String> = row.get("tags");

            let tags: Vec<String> = match tags_str {
                Some(s) if !s.trim().is_empty() => serde_json::from_str(&s).unwrap_or_default(),
                _ => Vec::new(),
            };

            let iso_date = published_at
                .split('T')
                .next()
                .unwrap_or(&published_at)
                .to_string();
            let display_date = if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&published_at) {
                dt.format("%B %d, %Y").to_string()
            } else {
                iso_date.clone()
            };

            let excerpt = crate::utils::html::extract_body_preview(&content).unwrap_or_else(|| {
                if content.len() > 160 {
                    format!("{}...", &content[..160])
                } else {
                    content.clone()
                }
            });

            posts.push(BlogPostItem {
                id,
                slug,
                title,
                excerpt,
                display_date,
                iso_date,
                tags,
            });
        }

        Ok(posts)
    }

    #[cfg(not(feature = "ssr"))]
    Ok(Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blog_post_item_serde() {
        let post = BlogPostItem {
            id: "uuid-123".to_string(),
            slug: "first-post".to_string(),
            title: "First Post".to_string(),
            excerpt: "This is a preview of the first post.".to_string(),
            display_date: "August 23, 2026".to_string(),
            iso_date: "2026-08-23".to_string(),
            tags: vec!["rust".to_string(), "wasm".to_string()],
        };

        let json = serde_json::to_string(&post).expect("Failed to serialize");
        let deserialized: BlogPostItem =
            serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(post, deserialized);
    }

    #[test]
    fn test_blog_post_item_tags_default() {
        let json = r#"{
            "id": "uuid-456",
            "slug": "tagless-post",
            "title": "Tagless Post",
            "excerpt": "No tags here.",
            "display_date": "August 23, 2026",
            "iso_date": "2026-08-23"
        }"#;

        let post: BlogPostItem = serde_json::from_str(json).expect("Failed to deserialize");
        assert!(post.tags.is_empty());
    }
}
