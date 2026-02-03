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
    use std::fs;
    use std::path::PathBuf;
    
    pub fn get_articles_dir() -> PathBuf {
        PathBuf::from("data/articles")
    }
    
    // Simple JWT verification helper
    // In a real app, this should be shared with backend logic
    pub fn verify_token(token: &str) -> Result<String, ServerFnError> {
        use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
        use serde::Deserialize;

        #[derive(Deserialize)]
        struct Claims {
            sub: String,
            exp: usize,
        }

        // WARN: Synchronize this secret with backend/src/api/admin.rs
        // Ideally, use an ENV var.
        let secret = b"change-this-secret-key-in-production-environment";
        
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret),
            &Validation::new(Algorithm::HS256),
        ).map_err(|_| ServerFnError::new("Invalid token"))?;
        
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
            if path.extension().map_or(false, |ext| ext == "json") {
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
    let safe_slug = article.slug.chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect::<String>()
        .to_lowercase();
        
    let path = dir.join(format!("{}.json", safe_slug));
    
    // If slug changed, we might want to handle renaming or deleting old... 
    // For now, let's assume slug updates create new files (and existing one remains until manual cleanup? 
    // Or better, assume UI doesn't allow editing slug easily, or if it does, it's a new post).
    // User requested "delete" button, so that handles old ones.
    
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
    use std::process::Command;
    
    verify_token(&token)?;

    let bucket = "gs://jakewray-portfolio/media/journalism/";
    let output = Command::new("gsutil")
        .arg("ls")
        .arg("-r") // Recursive
        .arg(bucket)
        .output()?;
    
    if !output.status.success() {
        return Err(ServerFnError::new("Failed to list GCS media"));
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut items = Vec::new();
    let base_url = "https://storage.googleapis.com/jakewray-portfolio";

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() || line.ends_with('/') { continue; } // Skip directories
        
        if let Some(path) = line.strip_prefix("gs://jakewray-portfolio/") {
            let name = path.split('/').last().unwrap_or(path).to_string();
            items.push(MediaItem {
                url: format!("{}/{}", base_url, path),
                name,
            });
        }
    }
    
    Ok(items)
}

#[server(UploadMedia, "/api")]
pub async fn upload_media(token: String, filename: String, data: Vec<u8>) -> Result<String, ServerFnError> {
    use self::ssr_utils::verify_token;
    use std::process::{Command, Stdio};
    use std::io::Write;
    
    verify_token(&token)?;

    // We'll upload to a 'uploads' folder for manual picking or sorting later
    let timestamp = chrono::Utc::now().timestamp();
    let safe_name = format!("{}_{}", timestamp, filename.replace(" ", "_"));
    let destination = format!("gs://jakewray-portfolio/media/journalism/uploads/{}", safe_name);
    
    let mut child = Command::new("gsutil")
        .arg("cp")
        .arg("-") // from stdin
        .arg(&destination)
        .stdin(Stdio::piped())
        .spawn()?;
    
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(&data)?;
    }
    
    let status = child.wait()?;
    if !status.success() {
        return Err(ServerFnError::new("Failed to upload to GCS"));
    }
    
    Ok(format!("https://storage.googleapis.com/jakewray-portfolio/media/journalism/uploads/{}", safe_name))
}
