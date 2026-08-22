use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

pub fn init_dummy_hash() {
    let _ = get_dummy_hash();
}

pub fn get_dummy_hash() -> &'static str {
    static DUMMY_HASH: OnceLock<String> = OnceLock::new();
    DUMMY_HASH.get_or_init(|| {
        let password = "dummy-password-that-will-never-match";
        let salt = SaltString::generate(&mut OsRng);
        get_argon2()
            .hash_password(password.as_bytes(), &salt)
            .expect("Failed to generate dummy hash")
            .to_string()
    })
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Serialize, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(sqlx::FromRow)]
pub struct UserRow {
    pub id: String,
    pub password_hash: String,
}

pub fn get_argon2() -> &'static Argon2<'static> {
    static ARGON2: OnceLock<Argon2<'static>> = OnceLock::new();
    ARGON2.get_or_init(|| {
        let params = argon2::Params::new(
            shared::auth::ARGON2_M_COST,
            shared::auth::ARGON2_T_COST,
            shared::auth::ARGON2_P_COST,
            Some(argon2::Params::DEFAULT_OUTPUT_LEN),
        )
        .expect("Valid Argon2 parameters");
        Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params)
    })
}

pub fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    get_argon2()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| e.to_string())
        .map(|hash| hash.to_string())
}

#[inline(never)]
pub fn verify_password(password: &str, password_hash: &str) -> bool {
    let parsed_hash = match PasswordHash::new(password_hash) {
        Ok(h) => h,
        Err(_) => {
            tracing::error!("Failed to parse password hash!");
            let dummy = get_dummy_hash();
            let parsed_dummy = PasswordHash::new(dummy).unwrap();
            let _ = get_argon2().verify_password(password.as_bytes(), &parsed_dummy);
            return false;
        }
    };
    get_argon2()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}
