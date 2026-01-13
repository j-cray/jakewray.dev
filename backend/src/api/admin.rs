use axum::{
    routing::{get, post},
    Router, Json,
    http::StatusCode,
    extract::State,
    response::IntoResponse,
};
use axum_extra::extract::cookie::{Cookie, SignedCookieJar};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use bcrypt::verify;
use std::sync::Arc;

pub fn router(state: crate::state::AppState) -> Router
{
    Router::new()
        .route("/login", post(login))
        .route("/logout", post(logout))
        .with_state(state)
}

#[derive(Deserialize)]
struct LoginPayload {
    username: String,
    password: String,
}

async fn login(
    State(pool): State<PgPool>,
    jar: SignedCookieJar,
    Json(payload): Json<LoginPayload>,
) ->  impl IntoResponse {
    let user_row = sqlx::query("SELECT id, password_hash FROM users WHERE username = $1")
        .bind(&payload.username)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()));

    match user_row {
        Ok(Some(row)) => {
            use sqlx::Row;
            let password_hash: String = row.try_get("password_hash").unwrap_or_default(); // Handle error properly in prod
            let id: i32 = row.try_get("id").unwrap_or_default();

            if verify(&payload.password, &password_hash).unwrap_or(false) {
                // Password correct
                let cookie = Cookie::build(("auth_token", id.to_string()))
                    .path("/")
                    .http_only(true)
                    .secure(true) // Should be true in prod
                    .same_site(axum_extra::extract::cookie::SameSite::Lax)
                    .build();

                (StatusCode::OK, jar.add(cookie))
            } else {
                (StatusCode::UNAUTHORIZED, jar)
            }
        }
        Ok(None) => (StatusCode::UNAUTHORIZED, jar),
        Err(e) => (e.0, jar),
    }
}

async fn logout(jar: SignedCookieJar) -> (StatusCode, SignedCookieJar) {
    (StatusCode::OK, jar.remove(Cookie::from("auth_token")))
}
