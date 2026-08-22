use leptos::prelude::*;
use serde::{Deserialize, Serialize};

// Re-exports for backward compatibility
#[cfg(feature = "ssr")]
pub use crate::api::auth::ssr_utils;
pub use crate::api::media::{
    delete_media, list_media, upload_media, DeleteMedia, ListMedia, MediaItem, UploadMedia,
};
pub use crate::utils::html::extract_figcaption;
pub use crate::utils::slug::{parse_article_date, sanitize_slug};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Article {
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
    #[serde(default)]
    pub status: Option<String>,
}

#[server(GetArticles, "/api")]
pub async fn get_articles() -> Result<Vec<Article>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use sqlx::SqlitePool;
        let pool = use_context::<SqlitePool>()
            .ok_or_else(|| ServerFnError::new("SqlitePool not found in Leptos context"))?;

        let rows = sqlx::query(
            "SELECT slug, title, excerpt, content, cover_image_url, cover_image_caption, author, published_at, status FROM articles WHERE (status IS NULL OR status = 'published' OR (status = 'scheduled' AND published_at <= strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))) AND status != 'draft' ORDER BY published_at DESC, title ASC, slug ASC"
        )
        .fetch_all(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Database query failed: {}", e)))?;

        let mut articles = Vec::new();
        for row in rows {
            use sqlx::Row;
            let slug: String = row.get("slug");
            let title: String = row.get("title");
            let content_html: String = row.get("content");
            let excerpt: Option<String> = row.get("excerpt");
            let cover_image_url: Option<String> = row.get("cover_image_url");
            let cover_image_caption: Option<String> = row.get("cover_image_caption");
            let author: String = row.get("author");
            let published_at: String = row.get("published_at");
            let status: Option<String> = row.get("status");

            // Format dates
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

            let mut images = Vec::new();
            if let Some(img) = cover_image_url {
                if !img.is_empty() {
                    images.push(img);
                }
            }

            let mut captions = Vec::new();
            if let Some(cap) = cover_image_caption {
                if !cap.is_empty() {
                    captions.push(cap);
                }
            }

            let excerpt_str = excerpt.unwrap_or_else(|| {
                crate::utils::html::extract_body_preview(&content_html).unwrap_or_default()
            });

            articles.push(Article {
                slug,
                title,
                iso_date,
                display_date,
                source_url: String::new(),
                content_html,
                images,
                captions,
                excerpt: excerpt_str,
                byline: if author.is_empty() {
                    None
                } else {
                    Some(author)
                },
                status,
            });
        }

        Ok(articles)
    }

    #[cfg(not(feature = "ssr"))]
    Ok(Vec::new())
}

#[server(GetArticle, "/api")]
pub async fn get_article(slug: String) -> Result<Option<Article>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use sqlx::SqlitePool;
        let pool = use_context::<SqlitePool>()
            .ok_or_else(|| ServerFnError::new("SqlitePool not found in Leptos context"))?;

        let clean_slug = sanitize_slug(&slug);

        let row = sqlx::query(
            "SELECT slug, title, excerpt, content, cover_image_url, cover_image_caption, author, published_at, status FROM articles WHERE slug = ?"
        )
        .bind(&clean_slug)
        .fetch_optional(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Database query failed: {}", e)))?;

        if let Some(row) = row {
            use sqlx::Row;
            let slug: String = row.get("slug");
            let title: String = row.get("title");
            let content_html: String = row.get("content");
            let excerpt: Option<String> = row.get("excerpt");
            let cover_image_url: Option<String> = row.get("cover_image_url");
            let cover_image_caption: Option<String> = row.get("cover_image_caption");
            let author: String = row.get("author");
            let published_at: String = row.get("published_at");
            let status: Option<String> = row.get("status");

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

            let mut images = Vec::new();
            if let Some(img) = cover_image_url {
                if !img.is_empty() {
                    images.push(img);
                }
            }

            let mut captions = Vec::new();
            if let Some(cap) = cover_image_caption {
                if !cap.is_empty() {
                    captions.push(cap);
                }
            }

            let excerpt_str = excerpt.unwrap_or_else(|| {
                crate::utils::html::extract_body_preview(&content_html).unwrap_or_default()
            });

            Ok(Some(Article {
                slug,
                title,
                iso_date,
                display_date,
                source_url: String::new(),
                content_html,
                images,
                captions,
                excerpt: excerpt_str,
                byline: if author.is_empty() {
                    None
                } else {
                    Some(author)
                },
                status,
            }))
        } else {
            Ok(None)
        }
    }

    #[cfg(not(feature = "ssr"))]
    Ok(None)
}

#[server(GetDraftsAndScheduled, "/api")]
pub async fn get_drafts_and_scheduled(token: String) -> Result<Vec<Article>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::api::auth::ssr_utils::verify_token;
        verify_token(&token)?;

        use sqlx::SqlitePool;
        let pool = use_context::<SqlitePool>()
            .ok_or_else(|| ServerFnError::new("SqlitePool not found in Leptos context"))?;

        let rows = sqlx::query(
            "SELECT slug, title, excerpt, content, cover_image_url, cover_image_caption, author, published_at, status FROM articles WHERE status = 'draft' OR (status = 'scheduled' AND published_at > strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) ORDER BY published_at DESC, title ASC, slug ASC"
        )
        .fetch_all(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Database query failed: {}", e)))?;

        let mut articles = Vec::new();
        for row in rows {
            use sqlx::Row;
            let slug: String = row.get("slug");
            let title: String = row.get("title");
            let content_html: String = row.get("content");
            let excerpt: Option<String> = row.get("excerpt");
            let cover_image_url: Option<String> = row.get("cover_image_url");
            let cover_image_caption: Option<String> = row.get("cover_image_caption");
            let author: String = row.get("author");
            let published_at: String = row.get("published_at");
            let status: Option<String> = row.get("status");

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

            let mut images = Vec::new();
            if let Some(img) = cover_image_url {
                if !img.is_empty() {
                    images.push(img);
                }
            }

            let mut captions = Vec::new();
            if let Some(cap) = cover_image_caption {
                if !cap.is_empty() {
                    captions.push(cap);
                }
            }

            let excerpt_str = excerpt.unwrap_or_else(|| {
                crate::utils::html::extract_body_preview(&content_html).unwrap_or_default()
            });

            articles.push(Article {
                slug,
                title,
                iso_date,
                display_date,
                source_url: String::new(),
                content_html,
                images,
                captions,
                excerpt: excerpt_str,
                byline: if author.is_empty() {
                    None
                } else {
                    Some(author)
                },
                status,
            });
        }

        Ok(articles)
    }

    #[cfg(not(feature = "ssr"))]
    Ok(Vec::new())
}

#[server(SaveArticle, "/api")]
pub async fn save_article(token: String, article: Article) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::api::auth::ssr_utils::verify_token;
        verify_token(&token)?;

        use sqlx::SqlitePool;
        let pool = use_context::<SqlitePool>()
            .ok_or_else(|| ServerFnError::new("SqlitePool not found in Leptos context"))?;

        let clean_slug = sanitize_slug(&article.slug);
        if clean_slug.is_empty() {
            return Err(ServerFnError::new("Article slug cannot be empty"));
        }

        let (published_at, _iso_date, _display_date) = parse_article_date(&article.display_date);
        let cover_image_url = article.images.first().cloned();
        let cover_image_caption = article.captions.first().cloned();
        let author = article.byline.unwrap_or_else(|| "Jake Wray".to_string());
        let status = article.status.unwrap_or_else(|| "published".to_string());

        let mut id = uuid::Uuid::new_v4().to_string();
        let existing = sqlx::query("SELECT id FROM articles WHERE slug = ?")
            .bind(&clean_slug)
            .fetch_optional(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Database query failed: {}", e)))?;

        if let Some(row) = existing {
            use sqlx::Row;
            id = row.get("id");
        }

        sqlx::query(
            "INSERT INTO articles (id, slug, title, excerpt, content, cover_image_url, cover_image_caption, author, published_at, origin, status, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 'local', ?, strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) \
             ON CONFLICT(slug) DO UPDATE SET \
                title = excluded.title, \
                excerpt = excluded.excerpt, \
                content = excluded.content, \
                cover_image_url = excluded.cover_image_url, \
                cover_image_caption = excluded.cover_image_caption, \
                author = excluded.author, \
                published_at = excluded.published_at, \
                status = excluded.status, \
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')",
        )
        .bind(&id)
        .bind(&clean_slug)
        .bind(&article.title)
        .bind(&article.excerpt)
        .bind(&article.content_html)
        .bind(&cover_image_url)
        .bind(&cover_image_caption)
        .bind(&author)
        .bind(&published_at)
        .bind(&status)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Database insert/update failed: {}", e)))?;

        Ok(())
    }

    #[cfg(not(feature = "ssr"))]
    Ok(())
}

#[server(DeleteArticle, "/api")]
pub async fn delete_article(token: String, slug: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::api::auth::ssr_utils::verify_token;
        verify_token(&token)?;

        use sqlx::SqlitePool;
        let pool = use_context::<SqlitePool>()
            .ok_or_else(|| ServerFnError::new("SqlitePool not found in Leptos context"))?;

        let clean_slug = sanitize_slug(&slug);

        sqlx::query("DELETE FROM articles WHERE slug = ?")
            .bind(&clean_slug)
            .execute(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Database delete failed: {}", e)))?;

        Ok(())
    }

    #[cfg(not(feature = "ssr"))]
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_article_status_deserialization() {
        let json_without_status = r#"{
            "slug": "test-post",
            "title": "Test Post",
            "iso_date": "2026-07-28",
            "display_date": "July 28, 2026",
            "source_url": "",
            "content_html": "<p>Hello</p>",
            "images": [],
            "captions": [],
            "excerpt": "Hello"
        }"#;

        let article: Article = serde_json::from_str(json_without_status).unwrap();
        assert_eq!(article.status, None);

        let json_with_status = r#"{
            "slug": "test-post",
            "title": "Test Post",
            "iso_date": "2026-07-28",
            "display_date": "July 28, 2026",
            "source_url": "",
            "content_html": "<p>Hello</p>",
            "images": [],
            "captions": [],
            "excerpt": "Hello",
            "status": "scheduled"
        }"#;

        let article_scheduled: Article = serde_json::from_str(json_with_status).unwrap();
        assert_eq!(article_scheduled.status, Some("scheduled".to_string()));
    }
}
