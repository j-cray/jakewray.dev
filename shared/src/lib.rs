use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Origin {
    Imported,
    Synced,
    Local,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    pub id: Uuid,
    pub wp_id: Option<i64>,
    pub slug: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub excerpt: Option<String>,
    pub content: String,
    pub cover_image_url: Option<String>,
    pub author: String,
    pub published_at: DateTime<Utc>,
    pub origin: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlogPost {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub content: String,
    pub published_at: DateTime<Utc>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MediaCategory {
    Photography,
    VisualArt,
    Video,
    JSchool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MediaContext {
    Personal,
    Professional,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaItem {
    pub id: Uuid,
    pub title: Option<String>,
    pub description: Option<String>,
    pub url: String,
    pub thumbnail_url: Option<String>,
    pub category: MediaCategory,
    pub context: MediaContext,
    pub taken_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CreativeType {
    Story,
    Novel,
    Poetry,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreativeWork {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub work_type: CreativeType,
    pub synopsis: Option<String>,
    pub content: Option<String>,
    pub status: String,
    pub published_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub github_url: Option<String>,
    pub demo_url: Option<String>,
    pub technologies: Option<Vec<String>>,
    pub stars: i32,
    pub is_featured: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCount {
    pub count: i64,
}
