use std::sync::OnceLock;

static JWT_SECRET: OnceLock<Vec<u8>> = OnceLock::new();

/// Required initialization: Must be called at application startup before `get_jwt_secret()` is used,
/// otherwise `get_jwt_secret()` will panic.
pub fn init_jwt_secret() {
    JWT_SECRET.get_or_init(|| {
        std::env::var("JWT_SECRET")
            .expect("JWT_SECRET environment variable must be set")
            .into_bytes()
    });
}

pub fn get_jwt_secret() -> &'static [u8] {
    JWT_SECRET.get().expect("JWT_SECRET not initialized")
}
