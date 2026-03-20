use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
fn main() {
    let mut password = String::new();
    std::io::stdin().read_line(&mut password).expect("Failed to read password");
    let password = password.trim_end();
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(password.as_bytes(), &salt).unwrap();
    println!("{}", hash);
}
