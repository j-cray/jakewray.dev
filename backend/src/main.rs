#![recursion_limit = "256"]
use axum::body::Body;
use axum::http::Request;
use axum::middleware::{self, Next};
use axum::{extract::State, Router};
use bytes::Bytes;
use dotenvy::dotenv;
use frontend::{App, Shell};
use futures_util::stream;
use futures_util::StreamExt;
use leptos::context::provide_context;
use leptos::prelude::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::net::SocketAddr;
use std::str::FromStr;
use tower::ServiceBuilder;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod state;

use crate::state::AppState;
use axum::response::{IntoResponse, Response as AxumResponse};
use tower::ServiceExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize JWT Secret early so it panics at startup if missing
    shared::auth::init_jwt_secret();
    crate::api::admin::init_dummy_hash();

    // Improved error handling for DATABASE_URL
    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL environment variable must be set")?;

    // Parse options and ensure database is created if it doesn't exist
    let connect_options = SqliteConnectOptions::from_str(&database_url)
        .map_err(|e| format!("Invalid DATABASE_URL: {}", e))?
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .busy_timeout(std::time::Duration::from_secs(5));

    // With WAL mode, SQLite allows concurrent readers, but all writers are still
    // serialized with a single write lock. Setting max_connections(5) helps with concurrent
    // reads. We explicitly set min_connections(1) to keep one connection warm
    // to avoid cold-start latency.
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .min_connections(1)
        .connect_with(connect_options)
        .await
        .map_err(|e| format!("Failed to create database pool: {}", e))?;

    // Run migrations
    sqlx::migrate!("../migrations")
        .run(&pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to run migrations: {}", e);
            e
        })?;

    let user_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await
        .unwrap_or((0,));
    if user_count.0 == 0 {
        tracing::warn!("=====================================================================");
        tracing::warn!("WARNING: The 'users' table is empty. No admin user exists.");
        tracing::warn!("Run './scripts/setup-dev.sh' or inject a seed migration to create one.");
        tracing::warn!("=====================================================================");
    }

    if std::env::var("ENVIRONMENT").as_deref() == Ok("production") {
        match std::env::var("TRUSTED_PROXY_IPS").as_deref() {
            Err(_) => panic!("TRUSTED_PROXY_IPS must be set in production. Otherwise, all users behind a proxy will share a single rate-limit bucket."),
            Ok(ips) if ips.trim().is_empty() => panic!("TRUSTED_PROXY_IPS is set but empty. This will cause all proxies to be untrusted, collapsing rate limits."),
            Ok(ips) => {
                let default_ips = ips.split(',').map(|s| s.trim()).filter(|s| !s.is_empty());
                let mut has_private = false;
                for ip_str in default_ips {
                    if let Ok(ip) = ip_str.parse::<std::net::IpAddr>() {
                        if ip.is_loopback() {
                            has_private = true;
                            break;
                        }
                        match ip {
                            std::net::IpAddr::V4(v4) => {
                                let octets = v4.octets();
                                if octets[0] == 10 || (octets[0] == 172 && (16..=31).contains(&octets[1])) || (octets[0] == 192 && octets[1] == 168) {
                                    has_private = true;
                                    break;
                                }
                            }
                            std::net::IpAddr::V6(v6) => {
                                if (v6.segments()[0] & 0xfe00) == 0xfc00 {
                                    has_private = true;
                                    break;
                                }
                            }
                        }
                    }
                }

                if has_private {
                    tracing::warn!("=====================================================================");
                    tracing::warn!("WARNING: TRUSTED_PROXY_IPS contains private (e.g., Docker bridge) IPs.");
                    tracing::warn!("Container IPs can change on restart. Rate limiting may fail open if these are incorrect.");
                    tracing::warn!("Please verify these IPs post-deploy or use a more robust mechanism like static IPs (--ip) or docker network inspect.");
                    tracing::warn!("=====================================================================");
                }
            }
        }
    }

    // Build LeptosOptions from environment/config
    let site_addr: SocketAddr = std::env::var("LEPTOS_SITE_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:3000".to_string())
        .parse()
        .map_err(|e| format!("Invalid LEPTOS_SITE_ADDR: {}", e))?;

    let leptos_options = LeptosOptions::builder()
        .output_name(
            std::env::var("LEPTOS_OUTPUT_NAME").unwrap_or_else(|_| "jakewray_ca".to_string()),
        )
        .site_pkg_dir(std::env::var("LEPTOS_SITE_PKG_DIR").unwrap_or_else(|_| "pkg".to_string()))
        .site_root(std::env::var("LEPTOS_SITE_ROOT").unwrap_or_else(|_| "target/site".to_string()))
        .site_addr(site_addr)
        .reload_port(
            std::env::var("LEPTOS_RELOAD_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3001),
        )
        .build();

    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    let app_state = AppState {
        leptos_options: leptos_options.clone(),
        pool: pool.clone(),
    };

    // Build the application router with all routes
    let app = Router::new()
        .nest("/", api::router(app_state.clone()))
        .leptos_routes_with_context(
            &app_state,
            routes,
            {
                let pool = app_state.pool.clone();
                let options = app_state.leptos_options.clone();
                move || {
                    provide_context(pool.clone());
                    provide_context(options.clone());
                }
            },
            Shell,
        )
        .fallback(file_and_error_handler)
        .layer(ServiceBuilder::new().layer(middleware::from_fn(inject_doctype)))
        .with_state(app_state);

    tracing::info!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}

async fn inject_doctype(
    req: Request<Body>,
    next: Next,
) -> Result<AxumResponse, axum::http::StatusCode> {
    let res = next.run(req).await;

    if let Some(content_type) = res.headers().get(axum::http::header::CONTENT_TYPE) {
        if let Ok(ct_str) = content_type.to_str() {
            if ct_str.contains("text/html") {
                let (parts, body) = res.into_parts();
                let prefix = stream::once(async {
                    Ok::<Bytes, axum::Error>(Bytes::from_static(b"<!DOCTYPE html>"))
                });
                let new_body = Body::from_stream(prefix.chain(body.into_data_stream()));
                let new_res = axum::http::Response::from_parts(parts, new_body);
                return Ok(new_res.into_response());
            }
        }
    }

    Ok(res.into_response())
}

async fn file_and_error_handler(
    State(state): State<AppState>,
    uri: axum::http::Uri,
    req: axum::extract::Request,
) -> AxumResponse {
    let root = state.leptos_options.site_root.clone();
    let res = get_static_file(uri, &root).await;

    if res.status() == axum::http::StatusCode::OK {
        res.into_response()
    } else {
        let handler = leptos_axum::render_app_to_stream_with_context(
            move || {
                provide_context(state.leptos_options.clone());
                provide_context(state.pool.clone());
            },
            Shell,
        );
        handler(req).await.into_response()
    }
}

async fn get_static_file(uri: axum::http::Uri, root: &str) -> AxumResponse {
    let uri_str = uri.to_string();
    let req = axum::extract::Request::builder()
        .uri(uri)
        .body(axum::body::Body::empty())
        .unwrap_or_else(|e| {
            tracing::error!("Failed to build request for static file {}: {}", uri_str, e);
            // Return a dummy request that will likely fail gracefully in ServeDir
            axum::extract::Request::builder()
                .uri("/")
                .body(axum::body::Body::empty())
                .unwrap()
        });

    // `ServeDir` implements `Service`
    match tower_http::services::ServeDir::new(root).oneshot(req).await {
        Ok(res) => res.into_response(),
        Err(err) => {
            tracing::error!("Error serving static file {}: {}", uri_str, err);
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Something went wrong: {}", err),
            )
                .into_response()
        }
    }
}
