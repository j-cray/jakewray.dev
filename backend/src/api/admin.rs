use axum::{
    routing::{get, post},
    Router, Json,
};
use sqlx::PgPool;

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/login", post(login))
        .route("/me", get(me))
}

async fn login() -> &'static str {
    "Login Placeholder"
}

async fn me() -> &'static str {
    "Admin User"
}
