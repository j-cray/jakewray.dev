use axum::extract::FromRef;
use leptos::LeptosOptions;
use sqlx::PgPool;

#[derive(Clone, FromRef)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub pool: PgPool,
}
