use std::sync::OnceLock;

static JWT_SECRET: OnceLock<Vec<u8>> = OnceLock::new();

/// Required initialization: Call early if you want to fail fast on startup,
/// but `get_jwt_secret` will also lazily initialize it.
pub fn init_jwt_secret() {
    let _ = get_jwt_secret();
}

pub fn get_jwt_secret() -> &'static [u8] {
    JWT_SECRET.get_or_init(|| {
        let secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| {
                panic!("JWT_SECRET environment variable must be set. If this is a frontend/WASM build, the 'ssr' feature may have been incorrectly enabled.");
            });
        if secret.len() < 32 {
            panic!("JWT_SECRET must be at least 32 characters long for security.");
        }
        secret.into_bytes()
    })
}
