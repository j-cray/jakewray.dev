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
    std::io::stdin().read_line(&mut password).expect("Failed to read password");
    let password = password.trim_end_matches('\n').trim_end_matches('\r');
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(password.as_bytes(), &salt).unwrap();
    println!("{}", hash);
}
