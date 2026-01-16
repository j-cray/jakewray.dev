use axum::{extract::State, Router};
use dotenvy::dotenv;
use frontend::App;
use leptos::context::provide_context;
<<<<<<< HEAD
use leptos_axum::generate_route_list;
use sqlx::postgres::PgPoolOptions;

use dotenvy::dotenv;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use frontend::{App, Shell};
=======
use leptos::prelude::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
>>>>>>> origin/main

mod api;
mod state;

use crate::state::AppState;
use axum::response::{IntoResponse, Response as AxumResponse};
use tower::ServiceExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    // ... tracing setup ...
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

<<<<<<< HEAD
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let mut pool = None;
    let mut retries = 5;

    while retries > 0 {
        match PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
        {
            Ok(p) => {
                pool = Some(p);
                break;
            }
            Err(e) => {
                tracing::warn!("Failed to connect to database: {}. Retrying in 2 seconds...", e);
                retries -= 1;
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        }
    }

    let pool = pool.expect("Failed to create pool after retries");
=======
    // Improved error handling for DATABASE_URL
    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL environment variable must be set")?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .map_err(|e| format!("Failed to create database pool: {}", e))?;
>>>>>>> origin/main

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
    let _routes = generate_route_list(App);

    // Generate a key for signing cookies. In production, this should be consistent.
    let key = axum_extra::extract::cookie::Key::generate();

    let app_state = AppState {
        leptos_options: leptos_options.clone(),
        pool: pool.clone(),
        key: key.clone(),
    };

<<<<<<< HEAD
    let options_clone = leptos_options.clone();
    let pool_clone = pool.clone();
    let key_clone = key.clone();
    let _leptos_handler = move |req: axum::extract::Request| async move {
        let handler = leptos_axum::render_app_to_stream_with_context(
             move || {
                 provide_context(options_clone.clone());
                 provide_context(pool_clone.clone());
                 provide_context(key_clone.clone());
             },
             Shell
        );
        let res: AxumResponse = handler(req).await.into_response();
        res
    };

    let options_clone_2 = leptos_options.clone();
    let pool_clone_2 = pool.clone();
    let key_clone_2 = key.clone();
    let server_fn_handler = move |req: axum::extract::Request| async move {
        let res: AxumResponse = leptos_axum::handle_server_fns_with_context(
            move || {
                provide_context(options_clone_2.clone());
                provide_context(pool_clone_2.clone());
                provide_context(key_clone_2.clone());
            },
            req
        ).await.into_response();
        res
    };

    let options_clone_3 = leptos_options.clone();
    let pool_clone_3 = pool.clone();
    let key_clone_3 = key.clone();
    let fallback_handler = move |uri: axum::http::Uri, req: axum::extract::Request| async move {
        let res: AxumResponse = file_and_error_handler(uri, options_clone_3.clone(), pool_clone_3, key_clone_3, req).await.into_response();
        res
    };

    let api_router = api::router(app_state.clone());
    let app = Router::new()
        .nest("/api", api_router)
        .route("/ping", get(|| async { "pong" }))
        .route("/api/*fn_name", post(server_fn_handler))
        .fallback(fallback_handler)
        .layer(tower_http::trace::TraceLayer::new_for_http());
=======
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
>>>>>>> origin/main

    tracing::info!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

async fn file_and_error_handler(
    State(state): State<AppState>,
    uri: axum::http::Uri,
<<<<<<< HEAD
    options: LeptosOptions,
    pool_clone: sqlx::PgPool,
    key_clone: axum_extra::extract::cookie::Key,
    req: axum::extract::Request
=======
    req: axum::extract::Request,
>>>>>>> origin/main
) -> AxumResponse {
    let root = state.leptos_options.site_root.clone();
    let res = get_static_file(uri, &root).await;

    if res.status() == axum::http::StatusCode::OK {
        res.into_response()
    } else {
        let handler = leptos_axum::render_app_to_stream_with_context(
            move || {
<<<<<<< HEAD
                provide_context(options.clone());
                provide_context(pool_clone.clone());
                provide_context(key_clone.clone());
            },
            Shell
=======
                provide_context(state.leptos_options.clone());
                provide_context(state.pool.clone());
            },
            App,
>>>>>>> origin/main
        );
        let res: AxumResponse = handler(req).await.into_response();
        res
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
