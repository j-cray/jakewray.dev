use axum::Router;
use sqlx::PgPool;

mod public;
mod admin;

pub fn router() -> Router<PgPool> {
    Router::new()
        .merge(public::router())
        .nest("/admin", admin::router())
}
