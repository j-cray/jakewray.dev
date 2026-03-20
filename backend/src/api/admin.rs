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

const DUMMY_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$75vBQ9LN4IAiHrViVOPI4w$L1wC8aj0h6PO/I8xVshCOB0TjOa9CTkfx8dIKA/0FVY";

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

// Pin Argon2 parameters (m=19456, t=2, p=1) to prevent timing discrepancies if defaults ever change.
const ARGON2_M_COST: u32 = 19456;
const ARGON2_T_COST: u32 = 2;
const ARGON2_P_COST: u32 = 1;

fn get_argon2() -> Argon2<'static> {
    let params = argon2::Params::new(
        ARGON2_M_COST,
        ARGON2_T_COST,
        ARGON2_P_COST,
        Some(argon2::Params::DEFAULT_OUTPUT_LEN),
    )
    .expect("Valid Argon2 parameters");
    Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params)
}

fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    get_argon2()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| e.to_string())
        .map(|hash| hash.to_string())
}

#[derive(Clone)]
pub struct TrustedProxyIpKeyExtractor;

impl tower_governor::key_extractor::KeyExtractor for TrustedProxyIpKeyExtractor {
    type Key = String;

    fn extract<T>(&self, req: &Request<T>) -> Result<Self::Key, tower_governor::GovernorError> {
        let peer_ip = req
            .extensions()
            .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
            .map(|ci| ci.0.ip());

        static TRUSTED_PROXY_IPS: OnceLock<Vec<std::net::IpAddr>> = OnceLock::new();
        let trusted_ips = TRUSTED_PROXY_IPS.get_or_init(|| {
            std::env::var("TRUSTED_PROXY_IPS")
                .unwrap_or_default()
                .split(',')
                .filter_map(|s| s.trim().parse().ok())
                .collect()
        });

        let is_trusted_proxy = peer_ip.is_some_and(|ip| trusted_ips.contains(&ip));

        if is_trusted_proxy {
            // Priority 1: X-Real-IP is checked first.
            // Some proxy configurations use X-Real-IP to explicitly pass the client IP, overriding XFF lists.
            if let Some(real_ip) = req.headers().get("X-Real-IP").and_then(|h| h.to_str().ok()) {
                if let Ok(parsed_ip) = real_ip.parse::<std::net::IpAddr>() {
                    return Ok(parsed_ip.to_string());
                }
            }
            // Priority 2: X-Forwarded-For
            if let Some(forwarded_for) = req
                .headers()
                .get("X-Forwarded-For")
                .and_then(|h| h.to_str().ok())
            {
                // We pick the rightmost IP (next_back) under the exact assumption that the trusted Nginx configuration
                // uses `proxy_add_x_forwarded_for`, which appends the connecting client's IP to the right.
                // NOTE: If intermediate proxies exist between Nginx and this backend that are NOT in TRUSTED_PROXY_IPS,
                // the rightmost IP will be the last untrusted proxy's IP, not the true client.
                if let Some(last_ip) = forwarded_for.split(',').next_back() {
                    if let Ok(parsed_ip) = last_ip.trim().parse::<std::net::IpAddr>() {
                        return Ok(parsed_ip.to_string());
                    }
                }
            }
            tracing::warn!(
                "TRUSTED_PROXY_IPS allowed proxy IP {}, but no valid X-Real-IP or X-Forwarded-For header was found. Rate limiting will apply to the proxy IP.",
                peer_ip.unwrap()
            );
        }

        peer_ip
            .map(|ip| ip.to_string())
            .ok_or(tower_governor::GovernorError::UnableToExtractKey)
    }
}

#[inline(never)]
fn verify_password(password: &str, password_hash: &str) -> bool {
    let parsed_hash = match PasswordHash::new(password_hash) {
        Ok(h) => h,
        Err(_) => return false,
    };
    get_argon2()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

pub fn router(state: crate::state::AppState) -> Router<crate::state::AppState> {
    // NOTE: tower_governor uses in-memory state. A server restart will reset all rate limit counters.
    // Burst windows completely refresh across restarts.
    let login_governor_config = std::sync::Arc::new(
        tower_governor::governor::GovernorConfigBuilder::default()
            .key_extractor(TrustedProxyIpKeyExtractor)
            .per_second(1)
            .burst_size(1)
            .finish()
            .unwrap(),
    );

    let login_governor_layer = tower_governor::GovernorLayer {
        config: login_governor_config,
    };

    let password_governor_config = std::sync::Arc::new(
        tower_governor::governor::GovernorConfigBuilder::default()
            .key_extractor(TrustedProxyIpKeyExtractor)
            .per_second(1)
            .burst_size(1)
            .finish()
            .unwrap(),
    );

    let password_governor_layer = tower_governor::GovernorLayer {
        config: password_governor_config,
    };

    let me_governor_layer = tower_governor::GovernorLayer {
        config: std::sync::Arc::new(
            tower_governor::governor::GovernorConfigBuilder::default()
                .key_extractor(TrustedProxyIpKeyExtractor)
                .per_second(5)
                .burst_size(10)
                .finish()
                .unwrap(),
        ),
    };

    Router::new()
        .route("/login", post(login).route_layer(login_governor_layer))
        .route(
            "/password",
            post(change_password).route_layer(password_governor_layer),
        )
        .route("/me", get(me).route_layer(me_governor_layer))
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

    let (hash_to_verify, is_valid_user) = match user {
        Some(ref u) => (u.password_hash.as_str(), true),
        None => {
            // To prevent early-return timing leaks, we always verify a password hash.
            // If the user doesn't exist, we use a dummy hash. The dummy hash's source
            // password is irrelevant as it's only used to consume time.
            (DUMMY_HASH, false)
        }
    };

    let password_match = verify_password(&req.password, hash_to_verify);
    let is_invalid = !is_valid_user || !password_match;

    if is_invalid {
        if content_type.contains("application/x-www-form-urlencoded")
            || content_type.contains("multipart/form-data")
            || accept.contains("text/html")
        {
            return Ok(Redirect::to("/admin/login?error=invalid").into_response());
        }

        return Err((StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()));
    }

    let exp = (Utc::now() + Duration::hours(24)).timestamp() as usize;
    let claims = Claims {
        sub: user.unwrap().id,
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
    let token_data = jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(shared::auth::get_jwt_secret()),
        &validation,
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?;

    Ok(Json(serde_json::json!({
        "authenticated": true,
        "sub": token_data.claims.sub
    })))
}

async fn change_password(
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
            "Current password length must be no more than 128 bytes (for Argon2 processing).".to_string(),
        ));
    }

    let char_count = req.new_password.chars().count();
    let byte_count = req.new_password.len();
    if char_count < 12 || byte_count > 128 {
        return Err((
            StatusCode::BAD_REQUEST,
            "New password length must be at least 12 characters and no more than 128 bytes (for Argon2 processing).".to_string(),
        ));
    }

    let user_id = uuid::Uuid::parse_str(&token_data.claims.sub).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Invalid user ID format in token".to_string(),
        )
    })?;

    // Verify current password
    let user: Option<UserRow> = sqlx::query_as("SELECT id, password_hash FROM users WHERE id = ?")
        .bind(user_id.to_string())
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
        None => (DUMMY_HASH, false),
    };

    let password_match = verify_password(&req.current_password, hash_to_verify);

    if !is_valid_user || !password_match {
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
        .bind(user_id.to_string())
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
