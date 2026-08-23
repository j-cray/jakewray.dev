#[cfg(feature = "ssr")]
pub mod ssr_utils {
    use leptos::prelude::ServerFnError;

    // Simple JWT verification helper
    pub fn verify_token(token: &str) -> Result<String, ServerFnError> {
        use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
        use serde::Deserialize;

        #[derive(Deserialize)]
        struct Claims {
            sub: String,
            #[allow(dead_code)]
            exp: usize,
        }

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(shared::auth::get_jwt_secret()),
            &Validation::new(Algorithm::HS256),
        )
        .map_err(|e| ServerFnError::new(format!("Invalid token: {}", e)))?;

        Ok(token_data.claims.sub)
    }
}
