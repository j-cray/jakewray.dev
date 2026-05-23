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
            _exp: usize,
        }

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(shared::auth::get_jwt_secret()),
            &Validation::new(Algorithm::HS256),
        )
        .map_err(|_| ServerFnError::new("Invalid token"))?;

        Ok(token_data.claims.sub)
    }
}

#[server(GetArticles, "/api")]
pub async fn get_articles() -> Result<Vec<Article>, ServerFnError> {
    use self::ssr_utils::get_articles_dir;
    use std::fs;

    let dir = get_articles_dir();
    let mut articles = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(article) = serde_json::from_str::<Article>(&content) {
                        articles.push(article);
                    }
                }
            }
        }
    }

    // Sort by date desc
    articles.sort_by(|a, b| b.iso_date.cmp(&a.iso_date));

    Ok(articles)
}

#[server(GetArticle, "/api")]
pub async fn get_article(slug: String) -> Result<Option<Article>, ServerFnError> {
    use self::ssr_utils::get_articles_dir;
    use std::fs;

    let path = get_articles_dir().join(format!("{}.json", slug));

    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(path)?;
    let article = serde_json::from_str(&content)?;

    Ok(Some(article))
}

#[server(SaveArticle, "/api")]
pub async fn save_article(token: String, article: Article) -> Result<(), ServerFnError> {
    use self::ssr_utils::{get_articles_dir, verify_token};
    use std::fs;

    verify_token(&token)?; // Guard

    let dir = get_articles_dir();
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }

    // Sanitize slug just in case
    let safe_slug = article
        .slug
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect::<String>()
        .to_lowercase();

    let path = dir.join(format!("{}.json", safe_slug));

    let content = serde_json::to_string_pretty(&article)?;
    fs::write(path, content)?;

    Ok(())
}

#[server(DeleteArticle, "/api")]
pub async fn delete_article(token: String, slug: String) -> Result<(), ServerFnError> {
    use self::ssr_utils::{get_articles_dir, verify_token};
    use std::fs;

    verify_token(&token)?; // Guard

    let path = get_articles_dir().join(format!("{}.json", slug));
    if path.exists() {
        fs::remove_file(path)?;
    }

    Ok(())
}

#[server(ListMedia, "/api")]
pub async fn list_media(token: String) -> Result<Vec<MediaItem>, ServerFnError> {
    use self::ssr_utils::verify_token;
    verify_token(&token)?;

    #[cfg(feature = "ssr")]
    {
        use google_cloud_storage::client::{Client, ClientConfig};
        use google_cloud_storage::http::objects::list::{ListObjectsRequest, Query};

        let config = ClientConfig::default()
            .with_auth()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to load GCS auth config: {}", e)))?;
        let client = Client::new(config);

        let request = ListObjectsRequest {
            bucket: "jakewray-portfolio".to_string(),
            query: Query {
                prefix: Some("media/journalism/".to_string()),
                ..Default::default()
            },
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
                let name = object.name.split('/').next_back().unwrap_or(&object.name).to_string();
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

    #[cfg(feature = "ssr")]
    {
        use google_cloud_storage::client::{Client, ClientConfig};
        use google_cloud_storage::http::objects::upload::{UploadObjectRequest, UploadType};
        use google_cloud_storage::http::objects::Object;

        let config = ClientConfig::default()
            .with_auth()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to load GCS auth config: {}", e)))?;
        let client = Client::new(config);

        let ext = filtered_name.split('.').next_back().unwrap_or("").to_lowercase();
        let content_type = match ext.as_str() {
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            "gif" => "image/gif",
            "webp" => "image/webp",
            "svg" => "image/svg+xml",
            "mp4" => "video/mp4",
            _ => "application/octet-stream",
        };

        let upload_type = UploadType::Simple(google_cloud_storage::http::objects::upload::Media {
            data: data.into(),
            content_type: content_type.to_string().into(),
        });

        let object_name = format!("media/journalism/uploads/{}", safe_name);
        let request = UploadObjectRequest {
            bucket: "jakewray-portfolio".to_string(),
            upload_type,
            metadata: Object {
                name: object_name.clone(),
                ..Default::default()
            },
            ..Default::default()
        };

        client
            .upload_object(&request)
            .await
            .map_err(|e| ServerFnError::new(format!("GCS upload failed: {}", e)))?;

        Ok(format!(
            "https://storage.googleapis.com/jakewray-portfolio/{}",
            object_name
        ))
    }

    #[cfg(not(feature = "ssr"))]
    Ok(String::new())
}
