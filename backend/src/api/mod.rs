use axum::http::Request;
use axum::Router;
use std::sync::OnceLock;

pub mod admin;
mod public;

static TRUSTED_PROXY_IPS: OnceLock<Vec<std::net::IpAddr>> = OnceLock::new();

pub fn init_trusted_proxies() {
    let ips = get_trusted_proxies();
    tracing::info!("Initialized TRUSTED_PROXY_IPS: {:?}", ips);
}

pub(crate) fn get_trusted_proxies() -> &'static Vec<std::net::IpAddr> {
    TRUSTED_PROXY_IPS.get_or_init(|| {
        std::env::var("TRUSTED_PROXY_IPS")
            .unwrap_or_default()
            .split(',')
            .filter_map(|s| {
                let trimmed = s.trim();
                if trimmed.is_empty() {
                    return None;
                }
                match trimmed.parse() {
                    Ok(ip) => Some(ip),
                    Err(e) => {
                        tracing::warn!(
                            "Invalid IP address in TRUSTED_PROXY_IPS '{}': {}",
                            trimmed,
                            e
                        );
                        None
                    }
                }
            })
            .collect()
    })
}

pub fn extract_client_ip(
    headers: &axum::http::HeaderMap,
    peer_ip: Option<std::net::IpAddr>,
) -> Option<String> {
    let trusted_ips = get_trusted_proxies();
    let is_trusted_proxy = peer_ip.is_some_and(|ip| trusted_ips.contains(&ip));

    if is_trusted_proxy {
        // Priority 1: X-Real-IP
        // SECURITY NOTE: We unconditionally trust X-Real-IP here because `is_trusted_proxy`
        // confirmed this request came from our trusted local reverse proxy. This behavior
        // assumes that Nginx is explicitly configured with `proxy_set_header X-Real-IP $remote_addr;`
        // to overwrite any potentially forged X-Real-IP header sent by the client.
        if let Some(real_ip) = headers.get("X-Real-IP").and_then(|h| h.to_str().ok()) {
            if let Ok(parsed_ip) = real_ip.trim().parse::<std::net::IpAddr>() {
                return Some(parsed_ip.to_string());
            }
        }

        // Priority 2: X-Forwarded-For
        if let Some(forwarded_for) = headers.get("X-Forwarded-For").and_then(|h| h.to_str().ok()) {
            // We pick the rightmost IP (next_back) under the exact assumption that the trusted Nginx configuration
            // uses `proxy_add_x_forwarded_for`, which appends the connecting peer's IP (the hop right before Nginx) to the right.
            // We pick the rightmost IP because that is the most trusted hop added by our reverse proxy, preventing client-side spoofing.
            // NOTE: This assumes Nginx is the ONLY intermediate proxy. Any CDN or external load balancer
            // will put its own IP rightmost, making all traffic share one rate limit bucket.
            if let Some(last_ip) = forwarded_for.split(',').next_back() {
                if let Ok(parsed_ip) = last_ip.trim().parse::<std::net::IpAddr>() {
                    if Some(parsed_ip) == peer_ip {
                        tracing::warn!("X-Forwarded-For rightmost IP {} matches the proxy peer IP. This usually indicates a CDN or external load balancer is stripping or improperly appending headers, collapsing all clients into one rate-limit bucket.", parsed_ip);
                        // fall through to peer_ip fallback below
                    } else {
                        tracing::debug!("Extracted client IP {} from X-Forwarded-For rightmost entry. Multi-hop proxies (e.g. Cloudflare) may cause all clients to share this IP.", parsed_ip);
                        return Some(parsed_ip.to_string());
                    }
                }
            }
        }
        tracing::warn!(
            "TRUSTED_PROXY_IPS allowed proxy IP {}, but no valid X-Real-IP or X-Forwarded-For header was found. Rate limiting will apply to the proxy IP.",
            peer_ip.unwrap()
        );
    }

    peer_ip.map(|ip| ip.to_string())
}

#[derive(Clone)]
pub struct TrustedProxyIpKeyExtractor;

impl tower_governor::key_extractor::KeyExtractor for TrustedProxyIpKeyExtractor {
    type Key = String;

    fn extract<T>(&self, req: &Request<T>) -> Result<Self::Key, tower_governor::GovernorError> {
        let connect_info = req
            .extensions()
            .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>();

        if connect_info.is_none() {
            tracing::error!("CRITICAL: ConnectInfo is missing from request extensions! This should never happen because `into_make_service_with_connect_info` is used in main.rs. Rate limiting will fail closed and return 500 errors.");
        }

        let peer_ip = connect_info.map(|ci| ci.0.ip());

        extract_client_ip(req.headers(), peer_ip)
            .ok_or(tower_governor::GovernorError::UnableToExtractKey)
    }
}

pub fn router(state: crate::state::AppState) -> Router<crate::state::AppState> {
    Router::new()
        .merge(public::router(state.clone()))
        .nest("/admin", admin::router(state.clone()))
}
