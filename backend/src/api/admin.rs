use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::body::to_bytes;
use axum::body::Body;
use axum::http::{header, Request};
use axum::response::Html;
use axum::response::IntoResponse;
use axum::response::Json;
use axum::response::Redirect;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Router,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use std::sync::OnceLock;

static JWT_SECRET: OnceLock<Vec<u8>> = OnceLock::new();

pub fn init_jwt_secret() {
    let secret = std::env::var("JWT_SECRET")
        .expect("JWT_SECRET environment variable must be set")
        .into_bytes();
    JWT_SECRET
        .set(secret)
        .expect("JWT_SECRET initialized twice");
}

fn get_jwt_secret() -> &'static [u8] {
    JWT_SECRET.get().expect("JWT_SECRET not initialized")
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    exp: usize,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    token: String,
}

#[derive(Serialize, Deserialize)]
pub struct ChangePasswordRequest {
    current_password: String,
    new_password: String,
}

#[derive(sqlx::FromRow)]
struct UserRow {
    id: String,
    password_hash: String,
}

fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| e.to_string())
        .map(|hash| hash.to_string())
}

fn verify_password(password: &str, password_hash: &str) -> bool {
    let parsed_hash = match PasswordHash::new(password_hash) {
        Ok(h) => h,
        Err(_) => return false,
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

pub fn router(state: crate::state::AppState) -> Router<crate::state::AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/password", post(change_password))
        .route("/me", get(me))
        .with_state(state)
}

async fn login(
    State(pool): State<SqlitePool>,
    req: Request<Body>,
) -> Result<axum::response::Response, (StatusCode, String)> {
    let (parts, body) = req.into_parts();
    let content_type = parts
        .headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let accept = parts
        .headers
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let bytes = to_bytes(body, 16 * 1024)
        .await
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid request body".to_string()))?;

    let req: LoginRequest = if content_type.contains("application/json") {
        serde_json::from_slice(&bytes)
            .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid JSON".to_string()))?
    } else if content_type.contains("application/x-www-form-urlencoded")
        || content_type.contains("multipart/form-data")
    {
        serde_urlencoded::from_bytes(&bytes)
            .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid form data".to_string()))?
    } else {
        return Err((
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "Unsupported content type".to_string(),
        ));
    };

    let user: Option<UserRow> =
        sqlx::query_as("SELECT id, password_hash FROM users WHERE username = ?")
            .bind(&req.username)
            .fetch_optional(&pool)
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database error".to_string(),
                )
            })?;

    let is_invalid = match user {
        Some(ref u) => !verify_password(&req.password, &u.password_hash),
        None => {
            static DUMMY_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$Ewiz6jCZu9NGQaAJtWRLqg$Fn5yB19PZG+eTq/f1oKbw+tsqvhwuAnMI3TpQCIg9vI";
            let _ = verify_password(&req.password, DUMMY_HASH);
            true
        }
    };

    if is_invalid {
        if content_type.contains("application/x-www-form-urlencoded")
            || content_type.contains("multipart/form-data")
            || accept.contains("text/html")
        {
            return Ok(Redirect::to("/admin/login?error=invalid").into_response());
        }

        return Err((StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()));
    }

    let user = user.expect("User should exist when credentials are valid");

    let exp = (Utc::now() + Duration::hours(24)).timestamp() as usize;
    let claims = Claims {
        sub: user.id.clone(),
        exp,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(get_jwt_secret()),
    )
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Token generation failed".to_string(),
        )
    })?;

    if content_type.contains("application/x-www-form-urlencoded")
        || content_type.contains("multipart/form-data")
        || accept.contains("text/html")
    {
        let html = format!(
            r#"<!DOCTYPE html><html lang="en"><head><meta charset="utf-8"><meta http-equiv="refresh" content="0; url=/admin/dashboard"></head><body><script>localStorage.setItem("admin_token","{}");location.replace("/admin/dashboard");</script></body></html>"#,
            token
        );
        Ok(Html(html).into_response())
    } else {
        Ok(Json(LoginResponse { token }).into_response())
    }
}

async fn me(headers: HeaderMap) -> Result<&'static str, StatusCode> {
    let token = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let validation = jsonwebtoken::Validation::default();
    jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(get_jwt_secret()),
        &validation,
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?;

    Ok("Authenticated")
}

async fn change_password(
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let token = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or((StatusCode::UNAUTHORIZED, "Missing token".to_string()))?;

    // Verify token (simple check, ideally decode claims)
    let validation = jsonwebtoken::Validation::default();
    let token_data = jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(get_jwt_secret()),
        &validation,
    )
    .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".to_string()))?;

    let user_id = token_data.claims.sub.parse::<uuid::Uuid>().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Invalid user ID in token".to_string(),
        )
    })?;

    // Verify current password
    let user: Option<UserRow> = sqlx::query_as("SELECT id, password_hash FROM users WHERE id = ?")
        .bind(user_id.to_string())
        .fetch_optional(&pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            )
        })?;

    let user = user.ok_or((StatusCode::NOT_FOUND, "User not found".to_string()))?;

    if !verify_password(&req.current_password, &user.password_hash) {
        return Err((
            StatusCode::FORBIDDEN,
            "Invalid current password".to_string(),
        ));
    }

    // Hash new password and update
    let new_hash = hash_password(&req.new_password).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to hash password".to_string(),
        )
    })?;

    sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
        .bind(new_hash)
        .bind(user_id.to_string())
        .execute(&pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database update failed".to_string(),
            )
        })?;

    Ok(StatusCode::OK)
}
