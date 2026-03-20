use std::sync::OnceLock;

static JWT_SECRET: OnceLock<Vec<u8>> = OnceLock::new();

pub fn init_jwt_secret() {
    let secret = std::env::var("JWT_SECRET")
        .expect("JWT_SECRET environment variable must be set")
        .into_bytes();
    JWT_SECRET
        .set(secret)
        .expect("JWT_SECRET initialized twice");
}

pub fn get_jwt_secret() -> &'static [u8] {
    JWT_SECRET.get().expect("JWT_SECRET not initialized")
}
