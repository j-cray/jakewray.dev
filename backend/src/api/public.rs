use axum::{
    routing::get,
    Router,
    Json,
    extract::State,
};
use sqlx::PgPool;
use shared::{Article, BlogPost};

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/health", get(health_check))
        .route("/articles", get(list_articles))
        .route("/blog", get(list_blog_posts))
}

async fn health_check() -> &'static str {
    "OK"
}

async fn list_articles(State(pool): State<PgPool>) -> Json<Vec<Article>> {
    let articles = sqlx::query_as!(
        Article,
        "SELECT id, wp_id, slug, title, subtitle, excerpt, content, cover_image_url, author, published_at, origin, created_at as _ignored, updated_at as _ignored_2 FROM articles ORDER BY published_at DESC LIMIT 20"
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default(); // Simple error handling for now

    Json(articles)
}

async fn list_blog_posts(State(pool): State<PgPool>) -> Json<Vec<BlogPost>> {
    let posts = sqlx::query_as!(
        BlogPost,
        "SELECT id, slug, title, content, published_at, tags FROM blog_posts ORDER BY published_at DESC LIMIT 20"
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    Json(posts)
}
