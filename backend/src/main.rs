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

    let conf = get_configuration(None).unwrap();
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
             move || {
                 provide_context(options_clone.clone());
             },
             App
        );
        handler(req).await.into_response()
    };

    let options_clone_2 = leptos_options.clone();
    let server_fn_handler = move |req: axum::extract::Request| async move {
        // Manually inject state
        leptos_axum::handle_server_fns(
            req
        ).await.into_response()
    };

    let options_clone_3 = leptos_options.clone();
    let fallback_handler = move |uri: axum::http::Uri, req: axum::extract::Request| async move {
        file_and_error_handler(uri, options_clone_3.clone(), req).await.into_response()
    };

    let app = Router::new()
        .merge(api::router(app_state.clone()))
        .route("/api/*fn_name", post(server_fn_handler))
        .route("/*path", get(leptos_handler))
        .fallback(fallback_handler);
        // .with_state(app_state) REMOVED: Router remains Router<()>

    tracing::info!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

async fn file_and_error_handler(
    uri: axum::http::Uri,
    options: LeptosOptions, // Changed from State<LeptosOptions>
    req: axum::extract::Request
) -> AxumResponse {
    let root = options.site_root.clone();
    let res = get_static_file(uri.clone(), &root).await;

    if res.status() == axum::http::StatusCode::OK {
        res.into_response()
    } else {
        let handler = leptos_axum::render_app_to_stream_with_context(
            move || {
                provide_context(options.clone());
            },
            App
        );
        handler(req).await.into_response()
    }
}

async fn get_static_file(uri: axum::http::Uri, root: &str) -> AxumResponse {
    let req = axum::extract::Request::builder()
        .uri(uri.clone())
        .body(axum::body::Body::empty())
        .unwrap();
    // `ServeDir` implements `Service`
    match tower_http::services::ServeDir::new(root).oneshot(req).await {
        Ok(res) => res.into_response(),
        Err(err) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", err),
        )
            .into_response(),
    }
}

