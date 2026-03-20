use argon2::{Algorithm, Version, Params, Argon2};

fn main() {
    let params = Params::new(19456, 2, 1, None).unwrap();
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    println!("OK");
}
