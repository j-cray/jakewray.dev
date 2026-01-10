use axum::{
    routing::get,
    Router,
    Json,
    extract::State,
};
use sqlx::PgPool;
use shared::{Article, BlogPost};

pub fn router<S>(state: S) -> Router
where
    S: Clone + Send + Sync + 'static,
    PgPool: axum::extract::FromRef<S>,
{
    Router::new()
        .route("/health", get(health_check))
        .route("/articles", get(list_articles))
        .route("/blog", get(list_blog_posts))
        .with_state(state)
}

async fn health_check() -> &'static str {
    "OK"
}

use sqlx::Row;

async fn list_articles(State(pool): State<PgPool>) -> Json<Vec<Article>> {
    let articles = sqlx::query("SELECT id, wp_id, slug, title, subtitle, excerpt, content, cover_image_url, author, published_at, origin FROM articles ORDER BY published_at DESC LIMIT 20")
        .map(|row: sqlx::postgres::PgRow| Article {
            id: row.get("id"),
            wp_id: row.get("wp_id"),
            slug: row.get("slug"),
            title: row.get("title"),
            subtitle: row.get("subtitle"),
            excerpt: row.get("excerpt"),
            content: row.get("content"),
            cover_image_url: row.get("cover_image_url"),
            author: row.get("author"),
            published_at: row.get("published_at"),
            origin: row.get("origin"),
        })
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

    Json(articles)
}

async fn list_blog_posts(State(pool): State<PgPool>) -> Json<Vec<BlogPost>> {
    let posts = sqlx::query("SELECT id, slug, title, content, published_at, tags FROM blog_posts ORDER BY published_at DESC LIMIT 20")
        .map(|row: sqlx::postgres::PgRow| BlogPost {
            id: row.get("id"),
            slug: row.get("slug"),
            title: row.get("title"),
            content: row.get("content"),
            published_at: row.get("published_at"),
            tags: row.get("tags"),
        })
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

    Json(posts)
}
