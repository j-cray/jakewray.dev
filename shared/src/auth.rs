use std::sync::OnceLock;

pub const ARGON2_M_COST: u32 = 19456;
pub const ARGON2_T_COST: u32 = 2;
pub const ARGON2_P_COST: u32 = 1;

#[cfg(feature = "ssr")]
static JWT_SECRET: OnceLock<Vec<u8>> = OnceLock::new();

/// Required initialization: Call early if you want to fail fast on startup,
/// but `get_jwt_secret` will also lazily initialize it.
#[cfg(feature = "ssr")]
pub fn init_jwt_secret() {
    let _ = get_jwt_secret();
}

#[cfg(feature = "ssr")]
pub fn get_jwt_secret() -> &'static [u8] {
    JWT_SECRET.get_or_init(|| {
        let secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| {
                panic!("JWT_SECRET environment variable must be set. If this is a frontend/WASM build, the 'ssr' feature may have been incorrectly enabled.");
            });
        if secret.len() < 32 {
            panic!("JWT_SECRET must be at least 32 bytes long for security.");
        }
        secret.into_bytes()
    })
}

/// Helper to check if a JWT token string has expired or is invalid without requiring the secret key.
/// Safe to use on both WASM client and server.
pub fn is_token_expired(token: &str) -> bool {
    let token = token.trim();
    if token.is_empty() {
        return true;
    }
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return true;
    }

    let payload_b64 = parts[1];
    let mut b64 = payload_b64.replace('-', "+").replace('_', "/");
    match b64.len() % 4 {
        2 => b64.push_str("=="),
        3 => b64.push('='),
        _ => {}
    }

    let decoded = match base64_decode_simple(&b64) {
        Some(bytes) => bytes,
        None => return true,
    };

    let claims: serde_json::Value = match serde_json::from_slice(&decoded) {
        Ok(v) => v,
        Err(_) => return true,
    };

    if let Some(exp) = claims.get("exp").and_then(|e: &serde_json::Value| e.as_i64()) {
        let now = get_current_timestamp();
        return now >= exp;
    }

    true
}

fn get_current_timestamp() -> i64 {
    #[cfg(target_arch = "wasm32")]
    {
        (js_sys::Date::now() / 1000.0) as i64
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        chrono::Utc::now().timestamp()
    }
}

fn base64_decode_simple(input: &str) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut buf: u32 = 0;
    let mut bits: u32 = 0;

    for byte in input.bytes() {
        let val = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' => break,
            b' ' | b'\n' | b'\r' | b'\t' => continue,
            _ => return None,
        } as u32;

        buf = (buf << 6) | val;
        bits += 6;

        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
            buf &= (1 << bits) - 1;
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_token_expired() {
        assert!(is_token_expired(""));
        assert!(is_token_expired("invalid.token"));

        // Payload with exp in past (exp: 1516239022 -> Jan 18 2018)
        // {"sub":"1234567890","name":"John Doe","iat":1516239022,"exp":1516239022}
        // base64url payload: eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyLCJleHAiOjE1MTYyMzkwMjJ9
        let expired_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyLCJleHAiOjE1MTYyMzkwMjJ9.signature";
        assert!(is_token_expired(expired_token));

        // Payload with exp in distant future (exp: 2524608000 -> Jan 1 2050)
        // {"sub":"1234567890","exp":2524608000}
        // payload json: {"sub":"1234567890","exp":2524608000}
        // base64url: eyJzdWIiOiIxMjM0NTY3ODkwIiwiZXhwIjoyNTI0NjA4MDAwfQ
        let future_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwiZXhwIjoyNTI0NjA4MDAwfQ.signature";
        assert!(!is_token_expired(future_token));
    }
}
