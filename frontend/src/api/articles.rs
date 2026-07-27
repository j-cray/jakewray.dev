use leptos::prelude::*;
use serde::{Deserialize, Serialize};

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
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MediaItem {
    pub url: String,
    pub name: String,
}

#[cfg(feature = "ssr")]
pub mod ssr_utils {
    use super::*;
    use std::path::PathBuf;

    pub fn get_articles_dir() -> PathBuf {
        PathBuf::from("data/articles")
    }

    // Simple JWT verification helper
    pub fn verify_token(token: &str) -> Result<String, ServerFnError> {
        use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
        use serde::Deserialize;

        #[derive(Deserialize)]
        struct Claims {
            sub: String,
            #[allow(dead_code)]
            exp: usize,
        }

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(shared::auth::get_jwt_secret()),
            &Validation::new(Algorithm::HS256),
        )
        .map_err(|e| ServerFnError::new(format!("Invalid token: {}", e)))?;

        Ok(token_data.claims.sub)
    }
}

pub fn extract_figcaption(content_html: &str) -> Option<String> {
    if let Some(start) = content_html.find("<figcaption") {
        if let Some(tag_end) = content_html[start..].find('>') {
            let content_start = start + tag_end + 1;
            if let Some(close_tag) = content_html[content_start..].find("</figcaption>") {
                let caption_text = content_html[content_start..content_start + close_tag].trim();
                if !caption_text.is_empty() {
                    return Some(caption_text.to_string());
                }
            }
        }
    }
    None
}

#[server(GetArticles, "/api")]
pub async fn get_articles() -> Result<Vec<Article>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use sqlx::SqlitePool;
        let pool = use_context::<SqlitePool>()
            .ok_or_else(|| ServerFnError::new("SqlitePool not found in Leptos context"))?;

        let rows = sqlx::query(
            "SELECT slug, title, excerpt, content, cover_image_url, cover_image_caption, author, published_at FROM articles ORDER BY published_at DESC"
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
            let published_at: String = row.get("published_at"); // text in sqlite

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

            let captions = if let Some(ref cap) = cover_image_caption {
                if !cap.trim().is_empty() {
                    vec![cap.clone()]
                } else if let Some(extracted) = extract_figcaption(&content_html) {
                    vec![extracted]
                } else {
                    Vec::new()
                }
            } else if let Some(extracted) = extract_figcaption(&content_html) {
                vec![extracted]
            } else {
                Vec::new()
            };

            articles.push(Article {
                slug,
                title,
                iso_date,
                display_date,
                source_url: String::new(),
                content_html,
                images,
                captions,
                excerpt: excerpt.unwrap_or_default(),
                byline: Some(author),
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

        let row = sqlx::query(
            "SELECT slug, title, excerpt, content, cover_image_url, cover_image_caption, author, published_at FROM articles WHERE slug = ?"
        )
        .bind(&slug)
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

            let captions = if let Some(ref cap) = cover_image_caption {
                if !cap.trim().is_empty() {
                    vec![cap.clone()]
                } else if let Some(extracted) = extract_figcaption(&content_html) {
                    vec![extracted]
                } else {
                    Vec::new()
                }
            } else if let Some(extracted) = extract_figcaption(&content_html) {
                vec![extracted]
            } else {
                Vec::new()
            };

            Ok(Some(Article {
                slug,
                title,
                iso_date,
                display_date,
                source_url: String::new(),
                content_html,
                images,
                captions,
                excerpt: excerpt.unwrap_or_default(),
                byline: Some(author),
            }))
        } else {
            Ok(None)
        }
    }

    #[cfg(not(feature = "ssr"))]
    Ok(None)
}

#[server(SaveArticle, "/api")]
pub async fn save_article(token: String, article: Article) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use self::ssr_utils::verify_token;
        verify_token(&token)?; // Guard
        use sqlx::SqlitePool;
        let pool = use_context::<SqlitePool>()
            .ok_or_else(|| ServerFnError::new("SqlitePool not found in Leptos context"))?;

        // Sanitize slug just in case
        let safe_slug = article
            .slug
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-' || *c == ',' || *c == ':' || *c == '.')
            .collect::<String>()
            .to_lowercase();

        let cover_image_url = article.images.first().cloned();
        let cover_image_caption = article.captions.first().cloned();
        let author = article.byline.unwrap_or_else(|| "Jake Wray".to_string());

        let published_at = if article.iso_date.contains('T') {
            article.iso_date.clone()
        } else {
            format!("{}T00:00:00.000Z", article.iso_date)
        };

        let mut id = uuid::Uuid::new_v4().to_string();

        let existing = sqlx::query("SELECT id FROM articles WHERE slug = ?")
            .bind(&safe_slug)
            .fetch_optional(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Database query failed: {}", e)))?;

        if let Some(row) = existing {
            use sqlx::Row;
            id = row.get("id");
        }

        sqlx::query(
            "INSERT INTO articles (id, slug, title, content, excerpt, cover_image_url, cover_image_caption, author, published_at, origin) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 'local') \
             ON CONFLICT(slug) DO UPDATE SET \
                title = excluded.title, \
                content = excluded.content, \
                excerpt = excluded.excerpt, \
                cover_image_url = excluded.cover_image_url, \
                cover_image_caption = excluded.cover_image_caption, \
                author = excluded.author, \
                published_at = excluded.published_at"
        )
        .bind(&id)
        .bind(&safe_slug)
        .bind(&article.title)
        .bind(&article.content_html)
        .bind(&article.excerpt)
        .bind(&cover_image_url)
        .bind(&cover_image_caption)
        .bind(&author)
        .bind(&published_at)
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
        use self::ssr_utils::verify_token;
        verify_token(&token)?; // Guard
        use sqlx::SqlitePool;
        let pool = use_context::<SqlitePool>()
            .ok_or_else(|| ServerFnError::new("SqlitePool not found in Leptos context"))?;

        sqlx::query("DELETE FROM articles WHERE slug = ?")
            .bind(&slug)
            .execute(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Database delete failed: {}", e)))?;

        Ok(())
    }

    #[cfg(not(feature = "ssr"))]
    Ok(())
}

#[server(ListMedia, "/api")]
pub async fn list_media(token: String) -> Result<Vec<MediaItem>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use self::ssr_utils::verify_token;
        verify_token(&token)?;
        use google_cloud_storage::client::{Client, ClientConfig};
        use google_cloud_storage::http::objects::list::ListObjectsRequest;

        let config = ClientConfig::default()
            .with_auth()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to load GCS auth config: {}", e)))?;
        let client = Client::new(config);

        let request = ListObjectsRequest {
            bucket: "jakewray-portfolio".to_string(),
            prefix: Some("media/journalism/".to_string()),
            ..Default::default()
        };

        let response = client
            .list_objects(&request)
            .await
            .map_err(|e| ServerFnError::new(format!("GCS list objects failed: {}", e)))?;

        let mut items = Vec::new();
        let base_url = "https://storage.googleapis.com/jakewray-portfolio";

        if let Some(objects) = response.items {
            for object in objects {
                let name = object
                    .name
                    .split('/')
                    .next_back()
                    .unwrap_or(&object.name)
                    .to_string();
                if name.is_empty() {
                    continue; // Skip directory placeholders
                }
                items.push(MediaItem {
                    url: format!("{}/{}", base_url, object.name),
                    name,
                });
            }
        }

        Ok(items)
    }

    #[cfg(not(feature = "ssr"))]
    Ok(Vec::new())
}

#[server(UploadMedia, "/api")]
pub async fn upload_media(
    token: String,
    filename: String,
    data: Vec<u8>,
) -> Result<String, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use self::ssr_utils::verify_token;
        verify_token(&token)?;

        // We'll upload to a 'uploads' folder for manual picking or sorting later
        let filtered_name: String = filename
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
            .collect();

        if filtered_name.is_empty() {
            return Err(ServerFnError::new("Invalid filename"));
        }

        let timestamp = chrono::Utc::now().timestamp();
        let safe_name = format!("{}_{}", timestamp, filtered_name);
        use google_cloud_storage::client::{Client, ClientConfig};
        use google_cloud_storage::http::objects::upload::{Media, UploadObjectRequest, UploadType};

        let config = ClientConfig::default()
            .with_auth()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to load GCS auth config: {}", e)))?;
        let client = Client::new(config);

        let ext = filtered_name
            .split('.')
            .next_back()
            .unwrap_or("")
            .to_lowercase();
        let content_type = match ext.as_str() {
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            "gif" => "image/gif",
            "webp" => "image/webp",
            "svg" => "image/svg+xml",
            "mp4" => "video/mp4",
            _ => "application/octet-stream",
        };

        let upload_type = UploadType::Simple(Media {
            name: format!("media/journalism/uploads/{}", safe_name).into(),
            content_length: Some(data.len() as u64),
            content_type: content_type.to_string().into(),
        });

        let request = UploadObjectRequest {
            bucket: "jakewray-portfolio".to_string(),
            ..Default::default()
        };

        // GCS upload_object takes &UploadObjectRequest, Body (Vec<u8>), and &UploadType
        client
            .upload_object(&request, data, &upload_type)
            .await
            .map_err(|e| ServerFnError::new(format!("GCS upload failed: {}", e)))?;

        Ok(format!(
            "https://storage.googleapis.com/jakewray-portfolio/media/journalism/uploads/{}",
            safe_name
        ))
    }

    #[cfg(not(feature = "ssr"))]
    Ok(String::new())
}

#[server(DeleteMedia, "/api")]
pub async fn delete_media(token: String, object_name: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use self::ssr_utils::verify_token;
        verify_token(&token)?;
        use google_cloud_storage::client::{Client, ClientConfig};
        use google_cloud_storage::http::objects::delete::DeleteObjectRequest;

        let config = ClientConfig::default()
            .with_auth()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to load GCS auth config: {}", e)))?;
        let client = Client::new(config);

        // Safety check to prevent deleting things outside of the public uploads directory
        if !object_name.starts_with("media/journalism/") {
            return Err(ServerFnError::new("Unauthorized directory access"));
        }

        let request = DeleteObjectRequest {
            bucket: "jakewray-portfolio".to_string(),
            object: object_name,
            ..Default::default()
        };

        client
            .delete_object(&request)
            .await
            .map_err(|e| ServerFnError::new(format!("GCS delete failed: {}", e)))?;

        Ok(())
    }

    #[cfg(not(feature = "ssr"))]
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_figcaption_valid() {
        let html = r#"<figure class="wp-caption"><img src="foo.jpg"/><figcaption class="wp-caption-text">Wildrose Leader photo</figcaption></figure>"#;
        assert_eq!(extract_figcaption(html), Some("Wildrose Leader photo".to_string()));
    }

    #[test]
    fn test_extract_figcaption_none() {
        let html = r#"<div><p>No caption here</p></div>"#;
        assert_eq!(extract_figcaption(html), None);
    }
}

