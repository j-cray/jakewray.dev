use crate::state::AppState;
use axum::extract::State;
use axum::response::{IntoResponse, Response as AxumResponse};
use frontend::Shell;
use leptos::context::provide_context;
use tower::ServiceExt;

pub async fn file_and_error_handler(
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

pub async fn get_static_file(uri: axum::http::Uri, root: &str) -> AxumResponse {
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
