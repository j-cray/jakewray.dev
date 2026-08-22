pub mod admin;
pub mod proxy;
pub mod public;

pub use proxy::init_trusted_proxies;

use axum::Router;

pub fn router(state: crate::state::AppState) -> Router<crate::state::AppState> {
    Router::new()
        .merge(public::router(state.clone()))
        .nest("/admin", admin::router(state.clone()))
}
