use axum::{
    routing::{get, post},
    Router,
};
use leptos::prelude::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use dotenvy::dotenv;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use frontend::App;

mod api;
mod state;

use crate::state::AppState;
use axum::response::{Response as AxumResponse, IntoResponse};
use tower::ServiceExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "debug".into()))
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

    let options_clone = leptos_options.clone();
    let leptos_handler = move |req: axum::extract::Request| async move {
        let handler = leptos_axum::render_app_to_stream_with_context(
             options_clone.clone(),
             move || {},
             App
        );
        // Wait, 0.7 signature is weird.
        // Let's assume render_app_to_stream(App) is correct for now based on error?
        // Error: takes 1 arg, supplied 2.
        // I supplied (options, App).
        // So it probably wants just (App) or just a closure?
        // But options must be provided somewhere.
        // If I use render_app_to_stream(App), where does it get options? From Context?
        // Context is usually set by middleware.
        // I am NOT using LeptosRoutes middleware here.
        // So I must provide options.
        // Maybe render_app_to_stream_with_context?
        // I'll try render_app_to_stream(App) first as hint suggested.
        // But I need to ensure options are used.
        let handler = leptos_axum::render_app_to_stream(App);
        handler(req).await.into_response()
    };

    let app = Router::new()
        .merge(api::router(app_state.clone()))
        .route("/api/*fn_name", post(leptos_axum::handle_server_fns))
        .route("/*path", get(leptos_handler))
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
    match tower_http::services::ServeDir::new(root).oneshot(req).await {
        Ok(res) => Ok(res.map(axum::body::Body::new)),
        Err(err) => Err((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", err),
        )),
    }
}
