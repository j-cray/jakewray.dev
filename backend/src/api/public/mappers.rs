use shared::{Article, BlogPost};
use sqlx::Row;

pub fn map_article_row(row: sqlx::sqlite::SqliteRow) -> Result<Article, sqlx::Error> {
    let origin_str: String = row.try_get("origin")?;
    let origin = match origin_str.as_str() {
        "imported" => shared::Origin::Imported,
        "synced" => shared::Origin::Synced,
        _ => shared::Origin::Local,
    };
    let id_str: String = row.try_get("id")?;
    let id = id_str
        .parse::<uuid::Uuid>()
        .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
    Ok(Article {
        id,
        wp_id: row.try_get("wp_id")?,
        slug: row.try_get("slug")?,
        title: row.try_get("title")?,
        subtitle: row.try_get("subtitle")?,
        excerpt: row.try_get("excerpt")?,
        content: row.try_get("content")?,
        cover_image_url: row.try_get("cover_image_url")?,
        author: row.try_get("author")?,
        published_at: parse_flexible_datetime(row.try_get("published_at")?)?,
        origin,
    })
}

pub fn map_blog_post_row(row: sqlx::sqlite::SqliteRow) -> Result<BlogPost, sqlx::Error> {
    let tags_str: Option<String> = row.try_get("tags")?;
    let tags = match tags_str {
        Some(s) => match serde_json::from_str(&s) {
            Ok(t) => Some(t),
            Err(e) => return Err(sqlx::Error::Decode(Box::new(e))),
        },
        None => None,
    };
    let id_str: String = row.try_get("id")?;
    let id = id_str
        .parse::<uuid::Uuid>()
        .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
    Ok(BlogPost {
        id,
        slug: row.try_get("slug")?,
        title: row.try_get("title")?,
        content: row.try_get("content")?,
        published_at: parse_flexible_datetime(row.try_get("published_at")?)?,
        tags,
    })
}

pub fn parse_flexible_datetime(
    dt_str: String,
) -> Result<chrono::DateTime<chrono::Utc>, sqlx::Error> {
    chrono::DateTime::parse_from_rfc3339(&dt_str)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(&dt_str, "%Y-%m-%d %H:%M:%S")
                .map(|ndt| ndt.and_utc())
        })
        .map_err(|e| sqlx::Error::Decode(Box::new(e)))
}
