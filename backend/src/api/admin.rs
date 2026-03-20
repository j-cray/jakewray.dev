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
    // Configure rate limit: 2 requests per second, up to 5 burst
    let login_governor_conf = std::sync::Arc::new(
        tower_governor::governor::GovernorConfigBuilder::default()
            .key_extractor(tower_governor::key_extractor::SmartIpKeyExtractor)
            .per_second(2)
            .burst_size(5)
            .finish()
            .unwrap(),
    );
    let login_governor_layer = tower_governor::GovernorLayer {
        config: login_governor_conf,
    };

    let password_governor_conf = std::sync::Arc::new(
        tower_governor::governor::GovernorConfigBuilder::default()
            .key_extractor(tower_governor::key_extractor::SmartIpKeyExtractor)
            .per_second(2)
            .burst_size(5)
            .finish()
            .unwrap(),
    );
    let password_governor_layer = tower_governor::GovernorLayer {
        config: password_governor_conf,
    };

    Router::new()
        .route("/login", post(login).route_layer(login_governor_layer))
        .route(
            "/password",
            post(change_password).route_layer(password_governor_layer),
        )
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
            .map_err(|e| {
                tracing::error!("Database error during login fetch: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database error".to_string(),
                )
            })?;

    let is_invalid = match user {
        Some(ref u) => !verify_password(&req.password, &u.password_hash),
        None => {
            static DUMMY_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$75vBQ9LN4IAiHrViVOPI4w$L1wC8aj0h6PO/I8xVshCOB0TjOa9CTkfx8dIKA/0FVY";
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
        &EncodingKey::from_secret(shared::auth::get_jwt_secret()),
    )
    .map_err(|e| {
        tracing::error!("Token generation failed: {}", e);
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

async fn me(headers: HeaderMap) -> Result<Json<serde_json::Value>, StatusCode> {
    let token = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
    let _token_data = jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(shared::auth::get_jwt_secret()),
        &validation,
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?;

    Ok(Json(serde_json::json!({"authenticated": true})))
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
    let validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
    let token_data = jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(shared::auth::get_jwt_secret()),
        &validation,
    )
    .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".to_string()))?;

    if req.new_password.chars().count() < 12 {
        return Err((StatusCode::BAD_REQUEST, "Password too short".to_string()));
    }

    let user_id = &token_data.claims.sub;

    // Verify current password
    let user: Option<UserRow> = sqlx::query_as("SELECT id, password_hash FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching user for password change: {}", e);
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
    let new_hash = hash_password(&req.new_password).map_err(|e| {
        tracing::error!("Failed to hash new password: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to hash password".to_string(),
        )
    })?;

    sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
        .bind(new_hash)
        .bind(user_id)
        .execute(&pool)
        .await
        .map_err(|e| {
            tracing::error!("Database update failed for password change: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database update failed".to_string(),
            )
        })?;

    Ok(StatusCode::OK)
}
