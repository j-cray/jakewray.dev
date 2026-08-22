use crate::api::public::mappers::{map_article_row, map_blog_post_row, parse_flexible_datetime};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use shared::{Article, BlogPost, PageContent};
use sqlx::{Row, SqlitePool};

#[derive(Deserialize)]
pub struct Pagination {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub before: Option<String>,
}

pub async fn health_check() -> &'static str {
    "OK"
}

pub async fn get_page_by_slug(
    State(pool): State<SqlitePool>,
    Path(slug): Path<String>,
) -> Result<Json<PageContent>, (StatusCode, String)> {
    let row = sqlx::query("SELECT id, slug, title, content, updated_at FROM pages WHERE slug = ?")
        .bind(&slug)
        .fetch_optional(&pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch page '{}': {}", slug, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            )
        })?;

    match row {
        Some(row) => {
            let id_str: String = row.try_get("id").map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Decode error: {}", e),
                )
            })?;
            let id = id_str.parse::<uuid::Uuid>().map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("UUID parse error: {}", e),
                )
            })?;
            let updated_at_str: Option<String> = row.try_get("updated_at").ok();
            let updated_at = updated_at_str.and_then(|s| parse_flexible_datetime(s).ok());

            Ok(Json(PageContent {
                id,
                slug: row.try_get("slug").map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Decode error: {}", e),
                    )
                })?,
                title: row.try_get("title").map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Decode error: {}", e),
                    )
                })?,
                content: row.try_get("content").map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Decode error: {}", e),
                    )
                })?,
                updated_at,
            }))
        }
        None => Err((StatusCode::NOT_FOUND, "Page not found".to_string())),
    }
}

pub async fn list_articles(
    State(pool): State<SqlitePool>,
    Query(query): Query<Pagination>,
) -> Result<Json<Vec<Article>>, (StatusCode, String)> {
    let limit = query.limit.unwrap_or(20).min(50);

    if query.before.is_some() && query.offset.is_some() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Cannot use 'before' and 'offset' together".to_string(),
        ));
    }

    let rows_res = if let Some(before) = query.before {
        let dt = chrono::DateTime::parse_from_rfc3339(&before)
            .map_err(|_| {
                (
                    StatusCode::BAD_REQUEST,
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
            return Err((StatusCode::BAD_REQUEST, "Offset too large".to_string()));
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
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            ))
        }
    }
}

pub async fn list_blog_posts(
    State(pool): State<SqlitePool>,
    Query(query): Query<Pagination>,
) -> Result<Json<Vec<BlogPost>>, (StatusCode, String)> {
    let limit = query.limit.unwrap_or(20).min(50);

    if query.before.is_some() && query.offset.is_some() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Cannot use 'before' and 'offset' together".to_string(),
        ));
    }

    let rows_res = if let Some(before) = query.before {
        let dt = chrono::DateTime::parse_from_rfc3339(&before)
            .map_err(|_| {
                (
                    StatusCode::BAD_REQUEST,
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
            return Err((StatusCode::BAD_REQUEST, "Offset too large".to_string()));
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
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            ))
        }
    }
}
