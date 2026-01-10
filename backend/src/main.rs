use axum::{
    routing::{get, post},
    Router,
};
use leptos::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use dotenvy::dotenv;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use frontend::App;

mod api;
mod state;

use axum::extract::FromRef;
use crate::state::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool");

    sqlx::migrate!("../migrations")
        .run(&pool)
        .await?;

    let conf = get_configuration(None).await.unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    let app_state = AppState {
        leptos_options: leptos_options.clone(),
        pool: pool.clone(),
    };

    let app = Router::new()
        .merge(api::router(app_state.clone())) // Mount API routes
        .route("/api/*fn_name", post(leptos_axum::handle_server_fns)) // Server Functions integration
        .leptos_routes(&app_state, routes, App)
        .fallback(file_and_error_handler)
        .with_state(app_state);

    tracing::info!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

async fn file_and_error_handler(uri: axum::http::Uri, axum::extract::State(options): axum::extract::State<leptos::LeptosOptions>, req: axum::extract::Request) -> AxumResponse {
    let root = options.site_root.clone();
    let res = get_static_file(uri.clone(), &root).await.unwrap();

    if res.status() == axum::http::StatusCode::OK {
        res.into_response()
    } else {
        let handler = leptos_axum::render_app_to_stream(options.to_owned(), App);
        handler(req).await.into_response()
    }
}

async fn get_static_file(uri: axum::http::Uri, root: &str) -> Result<axum::http::Response<axum::body::Body>, (axum::http::StatusCode, String)> {
    let req = axum::http::Request::builder().uri(uri.clone()).body(axum::body::Body::empty()).unwrap();
    // This is a simplified static file handler, usually tower-http ServeDir is used
    // But since we need to fallback to Leptos, we do this check.
    // For now, let's use tower_http::services::ServeDir if possible or just rely on leptos_axum default
    // Actually, let's implement the simpler check:
    match tower_http::services::ServeDir::new(root).oneshot(req).await {
        Ok(res) => Ok(res.map(axum::body::Body::new)),
        Err(err) => Err((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", err),
        )),
    }
}

use axum::response::{Response as AxumResponse, IntoResponse};
use tower::ServiceExt; // for oneshot
