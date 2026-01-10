use axum::Router;
use sqlx::PgPool;

mod admin;
mod public;

pub fn router<S>(state: S) -> Router
where
    S: Clone + Send + Sync + 'static,
    PgPool: axum::extract::FromRef<S>,
{
    Router::new()
        .merge(public::router(state.clone()))
        .nest("/admin", admin::router(state.clone()))
}
