use axum::Router;

mod admin;
mod public;

pub fn router(state: crate::state::AppState) -> Router<crate::state::AppState> {
    Router::new()
        .merge(public::router(state.clone()))
        .nest("/admin", admin::router(state.clone()))
}
