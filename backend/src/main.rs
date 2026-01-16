use axum::{extract::State, Router};
use dotenvy::dotenv;
use frontend::App;
use leptos::context::provide_context;
use leptos::prelude::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
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

    // Improved error handling for DATABASE_URL
    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL environment variable must be set")?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
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
            App,
        )
        .fallback(file_and_error_handler)
        .with_state(app_state);

    tracing::info!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
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
            App,
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
            panic!("Invalid request builder state");
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
