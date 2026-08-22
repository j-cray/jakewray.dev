use crate::api::admin::auth::{
    get_dummy_hash, hash_password, verify_password, ChangePasswordRequest, Claims, LoginRequest,
    LoginResponse, UserRow,
};
use axum::body::to_bytes;
use axum::body::Body;
use axum::extract::State;
use axum::http::{header, HeaderMap, Request, StatusCode};
use axum::response::{IntoResponse, Json, Response};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use sqlx::SqlitePool;

pub async fn login(
    State(pool): State<SqlitePool>,
    req: Request<Body>,
) -> Result<Response, (StatusCode, String)> {
    let (parts, body) = req.into_parts();
    let content_type = parts
        .headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let bytes = to_bytes(body, 16 * 1024)
        .await
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid request body".to_string()))?;

    if !content_type.contains("application/json") {
        return Err((
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "Unsupported content type".to_string(),
        ));
    }

    let req: LoginRequest = serde_json::from_slice(&bytes)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid JSON".to_string()))?;

    // Prevent extremely long passwords from exhausting Argon2 CPU time.
    if req.password.len() > 128 {
        return Err((StatusCode::BAD_REQUEST, "Password too long".to_string()));
    }

    if req.username.len() > 64 {
        return Err((StatusCode::BAD_REQUEST, "Username too long".to_string()));
    }

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

    let (hash_to_verify, is_valid_user) = match user {
        Some(ref u) => (u.password_hash.as_str(), true),
        None => {
            // To prevent early-return timing leaks, we always verify a password hash.
            // If the user doesn't exist, we use a dummy hash. The dummy hash's source
            // password is irrelevant as it's only used to consume time.
            (get_dummy_hash(), false)
        }
    };

    let hash = hash_to_verify.to_string();
    let pw = req.password.clone();
    let password_match = tokio::task::spawn_blocking(move || verify_password(&pw, &hash))
        .await
        .unwrap_or(false);
    let is_invalid = !is_valid_user || !password_match;

    if is_invalid {
        return Err((StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()));
    }

    let exp = (Utc::now() + Duration::hours(24)).timestamp() as usize;
    let claims = Claims {
        sub: user.expect("is_valid_user guarantees Some").id,
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

    Ok(Json(LoginResponse { token }).into_response())
}

pub async fn me(
    headers: HeaderMap,
    connect_info: Option<axum::extract::ConnectInfo<std::net::SocketAddr>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Design Note: The /me endpoint validates the JWT cryptographically but does not query the database.
    // This means a deleted user's JWT remains valid until expiration (24h). For a single-admin personal site,
    // this is an acceptable performance trade-off. `change_password` does perform a DB lookup.

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
    .map_err(|e| {
        let proxy_ip = connect_info
            .as_ref()
            .map(|ci| ci.0.ip().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let client_ip =
            crate::api::proxy::extract_client_ip(&headers, connect_info.map(|ci| ci.0.ip()))
                .unwrap_or_else(|| proxy_ip.clone());
        let safe_client_ip = client_ip.replace(['\n', '\r'], " ");

        tracing::warn!(
            "Invalid token on /me from client IP {} (via proxy {}): {}",
            safe_client_ip,
            proxy_ip,
            e
        );
        StatusCode::UNAUTHORIZED
    })?;

    Ok(Json(serde_json::json!({
        "authenticated": true
    })))
}

pub async fn change_password(
    State(pool): State<SqlitePool>,
    req: Request<Body>,
) -> Result<StatusCode, (StatusCode, String)> {
    let (parts, body) = req.into_parts();
    let content_type = parts
        .headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if !content_type.contains("application/json") {
        return Err((
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "Unsupported content type".to_string(),
        ));
    }

    let token = parts
        .headers
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

    let bytes = to_bytes(body, 16 * 1024)
        .await
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid request body".to_string()))?;

    let req: ChangePasswordRequest = serde_json::from_slice(&bytes)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid JSON".to_string()))?;

    let current_byte_count = req.current_password.len();
    if current_byte_count > 128 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Current password length must be no more than 128 bytes (for Argon2 processing)."
                .to_string(),
        ));
    }

    let byte_count = req.new_password.len();
    if !(12..=128).contains(&byte_count) {
        return Err((
            StatusCode::BAD_REQUEST,
            "New password length must be at least 12 bytes and no more than 128 bytes (policy limit).".to_string(),
        ));
    }

    let user_id_str = &token_data.claims.sub;
    if uuid::Uuid::parse_str(user_id_str).is_err() {
        tracing::error!("Valid JWT contained invalid UUID string: {}", user_id_str);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Invalid token payload".to_string(),
        ));
    }

    // Verify current password
    let user: Option<UserRow> = sqlx::query_as("SELECT id, password_hash FROM users WHERE id = ?")
        .bind(user_id_str)
        .fetch_optional(&pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching user for password change: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            )
        })?;

    let (hash_to_verify, is_valid_user) = match user {
        Some(ref u) => (u.password_hash.as_str(), true),
        None => (get_dummy_hash(), false),
    };

    let hash = hash_to_verify.to_string();
    let pw = req.current_password.clone();
    let password_match = tokio::task::spawn_blocking(move || verify_password(&pw, &hash))
        .await
        .unwrap_or(false);

    if !is_valid_user || !password_match {
        return Err((
            StatusCode::FORBIDDEN,
            "Invalid current password".to_string(),
        ));
    }

    // Hash new password and update
    let pw = req.new_password.clone();
    let new_hash = tokio::task::spawn_blocking(move || hash_password(&pw))
        .await
        .unwrap_or_else(|_| Err("Task join failed".to_string()))
        .map_err(|e| {
            tracing::error!("Failed to hash new password: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to hash password".to_string(),
            )
        })?;

    sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
        .bind(new_hash)
        .bind(user_id_str)
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
