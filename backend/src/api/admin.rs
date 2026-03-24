use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::body::to_bytes;
use axum::body::Body;
use axum::http::{header, Request};
use axum::response::IntoResponse;
use axum::response::Json;
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

pub fn init_dummy_hash() {
    let _ = get_dummy_hash();
}

fn get_dummy_hash() -> &'static str {
    static DUMMY_HASH: OnceLock<String> = OnceLock::new();
    DUMMY_HASH.get_or_init(|| {
        let password = "dummy-password-that-will-never-match";
        let salt = SaltString::generate(&mut OsRng);
        get_argon2()
            .hash_password(password.as_bytes(), &salt)
            .expect("Failed to generate dummy hash")
            .to_string()
    })
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

fn get_argon2() -> &'static Argon2<'static> {
    static ARGON2: OnceLock<Argon2<'static>> = OnceLock::new();
    ARGON2.get_or_init(|| {
        let params = argon2::Params::new(
            shared::auth::ARGON2_M_COST,
            shared::auth::ARGON2_T_COST,
            shared::auth::ARGON2_P_COST,
            Some(argon2::Params::DEFAULT_OUTPUT_LEN),
        )
        .expect("Valid Argon2 parameters");
        Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params)
    })
}

fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    get_argon2()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| e.to_string())
        .map(|hash| hash.to_string())
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
    // KNOWN LIMITATION: tower_governor uses in-memory state. A server restart will reset all rate limit counters.
    // Burst windows completely refresh across restarts. Therefore, the effective rate limiting
    // window ONLY covers uptime, not absolute calendar time. An attacker who can trigger or observe
    // restarts could reset their login throttle window. For a low-traffic personal site, this is an
    // acceptable trade-off to avoid the complexity of a distributed rate limiter like Redis. It is recommended
    // to pair this with an OS-level fail2ban or log-based alerting to compensate.
    tracing::info!("Initializing rate limiters. Warning: In-memory rate limiter state resets on restart. Frequent restarts may bypass burst limits.");
    let login_governor_layer = tower_governor::GovernorLayer {
        config: std::sync::Arc::new(
            tower_governor::governor::GovernorConfigBuilder::default()
                .key_extractor(crate::api::TrustedProxyIpKeyExtractor)
                .per_second(1)
                .burst_size(1)
                .finish()
                .unwrap(),
        ),
    };

    let password_governor_layer = tower_governor::GovernorLayer {
        config: std::sync::Arc::new(
            tower_governor::governor::GovernorConfigBuilder::default()
                .key_extractor(crate::api::TrustedProxyIpKeyExtractor)
                .per_second(1)
                .burst_size(1)
                .finish()
                .unwrap(),
        ),
    };

    let me_governor_layer = tower_governor::GovernorLayer {
        config: std::sync::Arc::new(
            tower_governor::governor::GovernorConfigBuilder::default()
                .key_extractor(crate::api::TrustedProxyIpKeyExtractor)
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

    let password_match = verify_password(&req.password, hash_to_verify);
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

async fn me(
    headers: HeaderMap,
    peer_info: Option<axum::extract::ConnectInfo<std::net::SocketAddr>>,
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
    let token_data = jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(shared::auth::get_jwt_secret()),
        &validation,
    )
    .map_err(|e| {
        let ip = peer_info
            .map(|ci| ci.0.ip().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        tracing::warn!("Invalid token on /me from {}: {}", ip, e);
        StatusCode::UNAUTHORIZED
    })?;

    Ok(Json(serde_json::json!({
        "authenticated": true,
        "user_id": token_data.claims.sub
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

    let user_id = uuid::Uuid::parse_str(&token_data.claims.sub).map_err(|e| {
        tracing::error!("Valid JWT contained invalid UUID string: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Invalid token payload".to_string(),
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
        None => (get_dummy_hash(), false),
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
