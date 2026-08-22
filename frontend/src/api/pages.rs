use leptos::prelude::*;
use serde::{Deserialize, Serialize};

pub use crate::utils::slug::sanitize_page_slug;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PageContent {
    pub slug: String,
    pub title: String,
    pub content_html: String,
    #[serde(default)]
    pub updated_at: Option<String>,
}

pub fn default_about_content() -> PageContent {
    PageContent {
        slug: "about".to_string(),
        title: "About Me".to_string(),
        content_html: "<p class=\"mb-6\">I am a journalist, developer, and photographer based in Northern British Columbia. I have a passion for uncovering stories that matter and documenting the world around me through both words and images.</p><p class=\"mb-6\">Currently, I am expanding my horizons into software development, building tools and applications that bridge the gap between storytelling and technology. This website itself is a testament to that journey—a work in progress where I explore new ideas and showcase my evolving portfolio.</p><h3 class=\"text-2xl font-semibold mt-8 mb-4 text-gray-800\">Journalism</h3><p class=\"mb-4\">My reporting focuses on community issues, Indigenous culture, and public interest stories in the Terrace and Kitimat regions. I believe in the power of local journalism to inform communities and hold power to account.</p><h3 class=\"text-2xl font-semibold mt-8 mb-4 text-gray-800\">Development</h3><p class=\"mb-4\">As a developer, I am interested in Rust, web technologies, and building efficient, user-focused applications. I am currently working on several projects that integrate my diverse interests.</p>".to_string(),
        updated_at: None,
    }
}

#[server(GetPage, "/api")]
pub async fn get_page(slug: String) -> Result<Option<PageContent>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use sqlx::SqlitePool;
        let pool = use_context::<SqlitePool>()
            .ok_or_else(|| ServerFnError::new("SqlitePool not found in Leptos context"))?;

        let clean_slug = sanitize_page_slug(&slug);

        let row = sqlx::query("SELECT slug, title, content, updated_at FROM pages WHERE slug = ?")
            .bind(&clean_slug)
            .fetch_optional(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Database query failed: {}", e)))?;

        if let Some(row) = row {
            use sqlx::Row;
            let slug: String = row.get("slug");
            let title: String = row.get("title");
            let content_html: String = row.get("content");
            let updated_at: Option<String> = row.get("updated_at");

            Ok(Some(PageContent {
                slug,
                title,
                content_html,
                updated_at,
            }))
        } else if clean_slug == "about" {
            // Fallback for resilient rendering if table hasn't been seeded yet
            Ok(Some(default_about_content()))
        } else {
            Ok(None)
        }
    }

    #[cfg(not(feature = "ssr"))]
    Ok(None)
}

#[server(SavePage, "/api")]
pub async fn save_page(token: String, page: PageContent) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::api::auth::ssr_utils::verify_token;
        verify_token(&token)?; // Admin guard

        use sqlx::SqlitePool;
        let pool = use_context::<SqlitePool>()
            .ok_or_else(|| ServerFnError::new("SqlitePool not found in Leptos context"))?;

        let clean_slug = sanitize_page_slug(&page.slug);
        if clean_slug.is_empty() {
            return Err(ServerFnError::new("Page slug cannot be empty"));
        }
        if page.title.trim().is_empty() {
            return Err(ServerFnError::new("Page title cannot be empty"));
        }

        let mut id = uuid::Uuid::new_v4().to_string();

        let existing = sqlx::query("SELECT id FROM pages WHERE slug = ?")
            .bind(&clean_slug)
            .fetch_optional(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Database query failed: {}", e)))?;

        if let Some(row) = existing {
            use sqlx::Row;
            id = row.get("id");
        }

        sqlx::query(
            "INSERT INTO pages (id, slug, title, content, updated_at) \
             VALUES (?, ?, ?, ?, strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) \
             ON CONFLICT(slug) DO UPDATE SET \
                title = excluded.title, \
                content = excluded.content, \
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')",
        )
        .bind(&id)
        .bind(&clean_slug)
        .bind(&page.title)
        .bind(&page.content_html)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Database insert/update failed: {}", e)))?;

        Ok(())
    }

    #[cfg(not(feature = "ssr"))]
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_about_content() {
        let about = default_about_content();
        assert_eq!(about.slug, "about");
        assert_eq!(about.title, "About Me");
        assert!(about.content_html.contains("Journalism"));
        assert!(about.content_html.contains("Development"));
        assert!(about.content_html.contains("Northern British Columbia"));
    }

    #[test]
    fn test_page_content_serde() {
        let original = PageContent {
            slug: "about".to_string(),
            title: "About Me".to_string(),
            content_html: "<p>Hello world</p>".to_string(),
            updated_at: Some("2026-08-21T12:00:00.000Z".to_string()),
        };

        let json = serde_json::to_string(&original).unwrap();
        let decoded: PageContent = serde_json::from_str(&json).unwrap();
        assert_eq!(original, decoded);
    }
}
