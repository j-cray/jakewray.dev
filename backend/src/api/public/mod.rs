pub mod handlers;
pub mod mappers;

use handlers::{get_page_by_slug, health_check, list_articles, list_blog_posts};

use axum::routing::get;
use axum::Router;

pub fn router(state: crate::state::AppState) -> Router<crate::state::AppState> {
    let public_governor_config = std::sync::Arc::new(
        tower_governor::governor::GovernorConfigBuilder::default()
            .key_extractor(crate::api::proxy::TrustedProxyIpKeyExtractor)
            .per_second(5)
            .burst_size(20)
            .finish()
            .unwrap(),
    );

    let articles_governor_layer = tower_governor::GovernorLayer {
        config: public_governor_config.clone(),
    };

    let blog_governor_layer = tower_governor::GovernorLayer {
        config: public_governor_config.clone(),
    };

    let pages_governor_layer = tower_governor::GovernorLayer {
        config: public_governor_config,
    };

    Router::new()
        .route("/health", get(health_check))
        .route(
            "/api/articles",
            get(list_articles).route_layer(articles_governor_layer),
        )
        .route(
            "/api/blog",
            get(list_blog_posts).route_layer(blog_governor_layer),
        )
        .route(
            "/api/pages/:slug",
            get(get_page_by_slug).route_layer(pages_governor_layer),
        )
        .with_state(state)
}
