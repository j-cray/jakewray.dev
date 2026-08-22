#![recursion_limit = "256"]
use axum::middleware;
use axum::Router;
use dotenvy::dotenv;
use frontend::{App, Shell};
use leptos::context::provide_context;
use leptos::prelude::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::net::SocketAddr;
use std::str::FromStr;
use tower::ServiceBuilder;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod server;
mod state;

use crate::server::{file_and_error_handler, inject_doctype};
use crate::state::AppState;

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
    crate::api::init_trusted_proxies();

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
        .route(
            "/api/*fn_name",
            axum::routing::post({
                let pool = app_state.pool.clone();
                let options = app_state.leptos_options.clone();
                move |req| {
                    let pool = pool.clone();
                    let options = options.clone();
                    leptos_axum::handle_server_fns_with_context(
                        move || {
                            provide_context(pool.clone());
                            provide_context(options.clone());
                        },
                        req,
                    )
                }
            })
            .get({
                let pool = app_state.pool.clone();
                let options = app_state.leptos_options.clone();
                move |req| {
                    let pool = pool.clone();
                    let options = options.clone();
                    leptos_axum::handle_server_fns_with_context(
                        move || {
                            provide_context(pool.clone());
                            provide_context(options.clone());
                        },
                        req,
                    )
                }
            }),
        )
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

#[cfg(test)]
mod tests {
    use super::*;
    use server_fn::ServerFn;

    #[test]
    fn test_server_fn_registration() {
        assert!(!frontend::api::articles::GetArticles::PATH.is_empty());
        assert!(!frontend::api::articles::GetArticle::PATH.is_empty());
        assert!(!frontend::api::articles::GetDraftsAndScheduled::PATH.is_empty());
        assert!(!frontend::api::articles::SaveArticle::PATH.is_empty());
        assert!(!frontend::api::articles::DeleteArticle::PATH.is_empty());
        assert!(!frontend::api::articles::ListMedia::PATH.is_empty());
        assert!(!frontend::api::articles::UploadMedia::PATH.is_empty());
        assert!(!frontend::api::articles::DeleteMedia::PATH.is_empty());
        assert!(!frontend::api::pages::GetPage::PATH.is_empty());
        assert!(!frontend::api::pages::SavePage::PATH.is_empty());
    }
}
