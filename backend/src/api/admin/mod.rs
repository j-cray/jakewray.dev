pub mod auth;
pub mod handlers;

pub use auth::init_dummy_hash;
use handlers::{change_password, login, me};

use axum::routing::{get, post};
use axum::Router;

pub fn router(state: crate::state::AppState) -> Router<crate::state::AppState> {
    // KNOWN LIMITATION: tower_governor uses in-memory state. A server restart will reset all rate limit counters.
    // Burst windows completely refresh across restarts. Therefore, the effective rate limiting
    // window ONLY covers uptime, not absolute calendar time. An attacker who can trigger or observe
    // restarts could reset their login throttle window. For a low-traffic personal site, this is an
    // acceptable trade-off to avoid the complexity of a distributed rate limiter like Redis. It is REQUIRED
    // to pair this with an OS-level fail2ban or log-based alerting to compensate for the login endpoint.
    tracing::info!("Initializing rate limiters. Warning: In-memory rate limiter state resets on restart. Frequent restarts may bypass burst limits.");
    let shared_auth_governor_config = std::sync::Arc::new(
        tower_governor::governor::GovernorConfigBuilder::default()
            .key_extractor(crate::api::proxy::TrustedProxyIpKeyExtractor)
            .per_second(1)
            .burst_size(1)
            .finish()
            .unwrap(),
    );

    let login_governor_layer = tower_governor::GovernorLayer {
        config: shared_auth_governor_config.clone(),
    };

    let password_governor_layer = tower_governor::GovernorLayer {
        config: shared_auth_governor_config,
    };

    let me_governor_layer = tower_governor::GovernorLayer {
        config: std::sync::Arc::new(
            tower_governor::governor::GovernorConfigBuilder::default()
                .key_extractor(crate::api::proxy::TrustedProxyIpKeyExtractor)
                .per_second(5)
                .burst_size(10)
                .finish()
                .unwrap(),
        ),
    };

    Router::new()
        .route("/login", post(login).route_layer(login_governor_layer))
        .route(
            "/password",
            post(change_password).route_layer(password_governor_layer),
        )
        .route("/me", get(me).route_layer(me_governor_layer))
        .with_state(state)
}
