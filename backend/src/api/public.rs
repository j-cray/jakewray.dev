use axum::{extract::Path, extract::Query, extract::State, routing::get, Json, Router};
use shared::{Article, BlogPost, PageContent};
use sqlx::SqlitePool;

#[derive(serde::Deserialize)]
pub struct Pagination {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub before: Option<String>,
}
pub fn router(state: crate::state::AppState) -> Router<crate::state::AppState> {
    let public_governor_config = std::sync::Arc::new(
        tower_governor::governor::GovernorConfigBuilder::default()
            .key_extractor(crate::api::TrustedProxyIpKeyExtractor)
            .per_second(5)
            .burst_size(20)
            .finish()
            .unwrap(),
    );

    let articles_governor_layer = tower_governor::GovernorLayer {
        config: public_governor_config.clone(),
    };

    let blog_governor_layer = tower_governor::GovernorLayer {
        config: public_governor_config.clone(),
    };

    let pages_governor_layer = tower_governor::GovernorLayer {
        config: public_governor_config,
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
        .route(
            "/api/pages/:slug",
            get(get_page_by_slug).route_layer(pages_governor_layer),
        )
        .with_state(state)
}

async fn health_check() -> &'static str {
    "OK"
}

async fn get_page_by_slug(
    State(pool): State<SqlitePool>,
    Path(slug): Path<String>,
) -> Result<Json<PageContent>, (axum::http::StatusCode, String)> {
    let row = sqlx::query("SELECT id, slug, title, content, updated_at FROM pages WHERE slug = ?")
        .bind(&slug)
        .fetch_optional(&pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch page '{}': {}", slug, e);
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            )
        })?;

    match row {
        Some(row) => {
            let id_str: String = row.try_get("id").map_err(|e| {
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Decode error: {}", e),
                )
            })?;
            let id = id_str.parse::<uuid::Uuid>().map_err(|e| {
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    format!("UUID parse error: {}", e),
                )
            })?;
            let updated_at_str: Option<String> = row.try_get("updated_at").ok();
            let updated_at = updated_at_str.and_then(|s| parse_flexible_datetime(s).ok());

            Ok(Json(PageContent {
                id,
                slug: row.try_get("slug").map_err(|e| {
                    (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Decode error: {}", e),
                    )
                })?,
                title: row.try_get("title").map_err(|e| {
                    (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Decode error: {}", e),
                    )
                })?,
                content: row.try_get("content").map_err(|e| {
                    (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Decode error: {}", e),
                    )
                })?,
                updated_at,
            }))
        }
        None => Err((
            axum::http::StatusCode::NOT_FOUND,
            "Page not found".to_string(),
        )),
    }
}

use sqlx::Row;

async fn list_articles(
    State(pool): State<SqlitePool>,
    Query(query): Query<Pagination>,
) -> Result<Json<Vec<Article>>, (axum::http::StatusCode, String)> {
    let limit = query.limit.unwrap_or(20).min(50);

    if query.before.is_some() && query.offset.is_some() {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            "Cannot use 'before' and 'offset' together".to_string(),
        ));
    }

    let rows_res = if let Some(before) = query.before {
        let dt = chrono::DateTime::parse_from_rfc3339(&before)
            .map_err(|_| {
                (
                    axum::http::StatusCode::BAD_REQUEST,
                    "Invalid 'before' date format".to_string(),
                )
            })?
            .to_utc();
        let normalized = dt.format("%Y-%m-%dT%H:%M:%3fZ").to_string();
        sqlx::query("SELECT id, wp_id, slug, title, subtitle, excerpt, content, cover_image_url, author, published_at, origin FROM articles WHERE (status IS NULL OR status = 'published' OR (status = 'scheduled' AND published_at <= strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))) AND status != 'draft' AND published_at < ? ORDER BY published_at DESC LIMIT ?")
            .bind(normalized)
            .bind(limit)
            .try_map(map_article_row)
            .fetch_all(&pool)
            .await
    } else {
        let offset = query.offset.unwrap_or(0);
        if offset > 10_000 {
            return Err((
                axum::http::StatusCode::BAD_REQUEST,
                "Offset too large".to_string(),
            ));
        }
        sqlx::query("SELECT id, wp_id, slug, title, subtitle, excerpt, content, cover_image_url, author, published_at, origin FROM articles WHERE (status IS NULL OR status = 'published' OR (status = 'scheduled' AND published_at <= strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))) AND status != 'draft' ORDER BY published_at DESC LIMIT ? OFFSET ?")
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
            Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            ))
        }
    }
}

async fn list_blog_posts(
    State(pool): State<SqlitePool>,
    Query(query): Query<Pagination>,
) -> Result<Json<Vec<BlogPost>>, (axum::http::StatusCode, String)> {
    let limit = query.limit.unwrap_or(20).min(50);

    if query.before.is_some() && query.offset.is_some() {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            "Cannot use 'before' and 'offset' together".to_string(),
        ));
    }

    let rows_res = if let Some(before) = query.before {
        let dt = chrono::DateTime::parse_from_rfc3339(&before)
            .map_err(|_| {
                (
                    axum::http::StatusCode::BAD_REQUEST,
                    "Invalid 'before' date format".to_string(),
                )
            })?
            .to_utc();
        let normalized = dt.format("%Y-%m-%dT%H:%M:%3fZ").to_string();
        sqlx::query("SELECT id, slug, title, content, published_at, tags FROM blog_posts WHERE published_at < ? ORDER BY published_at DESC LIMIT ?")
            .bind(normalized)
            .bind(limit)
            .try_map(map_blog_post_row)
            .fetch_all(&pool)
            .await
    } else {
        let offset = query.offset.unwrap_or(0);
        if offset > 10_000 {
            return Err((
                axum::http::StatusCode::BAD_REQUEST,
                "Offset too large".to_string(),
            ));
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
            Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            ))
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
        published_at: parse_flexible_datetime(row.try_get("published_at")?)?,
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
        published_at: parse_flexible_datetime(row.try_get("published_at")?)?,
        tags,
    })
}

fn parse_flexible_datetime(dt_str: String) -> Result<chrono::DateTime<chrono::Utc>, sqlx::Error> {
    chrono::DateTime::parse_from_rfc3339(&dt_str)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(&dt_str, "%Y-%m-%d %H:%M:%S")
                .map(|ndt| ndt.and_utc())
        })
        .map_err(|e| sqlx::Error::Decode(Box::new(e)))
}
