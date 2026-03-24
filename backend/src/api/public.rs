use axum::{extract::Query, extract::State, routing::get, Json, Router};
use shared::{Article, BlogPost};
use sqlx::SqlitePool;

#[derive(serde::Deserialize)]
pub struct Pagination {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub before: Option<String>,
}
pub fn router(state: crate::state::AppState) -> Router<crate::state::AppState> {
    let common_governor_config = std::sync::Arc::new(
        tower_governor::governor::GovernorConfigBuilder::default()
            .key_extractor(crate::api::TrustedProxyIpKeyExtractor)
            .per_second(5)
            .burst_size(20)
            .finish()
            .unwrap(),
    );

    let articles_governor_layer = tower_governor::GovernorLayer {
        config: common_governor_config.clone(),
    };

    let blog_governor_layer = tower_governor::GovernorLayer {
        config: common_governor_config.clone(),
    };

    Router::new()
        .route("/health", get(health_check))
        .route(
            "/api/articles",
            get(list_articles).route_layer(articles_governor_layer),
        )
        .route(
            "/api/blog",
            get(list_blog_posts).route_layer(blog_governor_layer),
        )
        .with_state(state)
}

async fn health_check() -> &'static str {
    "OK"
}

use sqlx::Row;

async fn list_articles(
    State(pool): State<SqlitePool>,
    Query(query): Query<Pagination>,
) -> Result<Json<Vec<Article>>, axum::http::StatusCode> {
    let limit = query.limit.unwrap_or(20).min(50);

    if query.before.is_some() && query.offset.is_some() {
        return Err(axum::http::StatusCode::BAD_REQUEST);
    }

    let rows_res = if let Some(before) = query.before {
        if chrono::DateTime::parse_from_rfc3339(&before).is_err() {
            return Err(axum::http::StatusCode::BAD_REQUEST);
        }
        sqlx::query("SELECT id, wp_id, slug, title, subtitle, excerpt, content, cover_image_url, author, published_at, origin FROM articles WHERE published_at < ? ORDER BY published_at DESC LIMIT ?")
            .bind(before)
            .bind(limit)
            .try_map(map_article_row)
            .fetch_all(&pool)
            .await
    } else {
        let offset = query.offset.unwrap_or(0);
        if offset > 10_000 {
            return Err(axum::http::StatusCode::BAD_REQUEST);
        }
        sqlx::query("SELECT id, wp_id, slug, title, subtitle, excerpt, content, cover_image_url, author, published_at, origin FROM articles ORDER BY published_at DESC LIMIT ? OFFSET ?")
            .bind(limit)
            .bind(offset)
            .try_map(map_article_row)
            .fetch_all(&pool)
            .await
    };

    match rows_res {
        Ok(articles) => Ok(Json(articles)),
        Err(e) => {
            tracing::error!("Failed to fetch articles: {}", e);
            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn list_blog_posts(
    State(pool): State<SqlitePool>,
    Query(query): Query<Pagination>,
) -> Result<Json<Vec<BlogPost>>, axum::http::StatusCode> {
    let limit = query.limit.unwrap_or(20).min(50);

    if query.before.is_some() && query.offset.is_some() {
        return Err(axum::http::StatusCode::BAD_REQUEST);
    }

    let rows_res = if let Some(before) = query.before {
        if chrono::DateTime::parse_from_rfc3339(&before).is_err() {
            return Err(axum::http::StatusCode::BAD_REQUEST);
        }
        sqlx::query("SELECT id, slug, title, content, published_at, tags FROM blog_posts WHERE published_at < ? ORDER BY published_at DESC LIMIT ?")
            .bind(before)
            .bind(limit)
            .try_map(map_blog_post_row)
            .fetch_all(&pool)
            .await
    } else {
        let offset = query.offset.unwrap_or(0);
        if offset > 10_000 {
            return Err(axum::http::StatusCode::BAD_REQUEST);
        }
        sqlx::query("SELECT id, slug, title, content, published_at, tags FROM blog_posts ORDER BY published_at DESC LIMIT ? OFFSET ?")
            .bind(limit)
            .bind(offset)
            .try_map(map_blog_post_row)
            .fetch_all(&pool)
            .await
    };

    match rows_res {
        Ok(posts) => Ok(Json(posts)),
        Err(e) => {
            tracing::error!("Failed to fetch blog posts: {}", e);
            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

fn map_article_row(row: sqlx::sqlite::SqliteRow) -> Result<Article, sqlx::Error> {
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
        published_at: row.try_get("published_at")?,
        origin,
    })
}

fn map_blog_post_row(row: sqlx::sqlite::SqliteRow) -> Result<BlogPost, sqlx::Error> {
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
        published_at: row.try_get("published_at")?,
        tags,
    })
}
