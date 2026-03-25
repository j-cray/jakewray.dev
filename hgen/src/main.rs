// Usage:
// This utility reads the password from standard input (stdin), not from command-line arguments.
// This improves security by preventing the password from appearing in shell history or `ps` output.
//
// Example:
// echo -n "mypassword" | cargo run --bin hgen
// or run `cargo run --bin hgen` and type the password followed by Enter.

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
fn main() {
    let mut password = String::new();
    std::io::Read::read_to_string(&mut std::io::stdin(), &mut password)
        .expect("Failed to read password");
    let password = password.trim_end_matches(['\r', '\n']);
    let salt = SaltString::generate(&mut OsRng);
    let params = argon2::Params::new(
        shared::auth::ARGON2_M_COST,
        shared::auth::ARGON2_T_COST,
        shared::auth::ARGON2_P_COST,
        Some(argon2::Params::DEFAULT_OUTPUT_LEN),
    )
    .unwrap();
    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
    let hash = argon2.hash_password(password.as_bytes(), &salt).unwrap();
    println!("{}", hash);
}
