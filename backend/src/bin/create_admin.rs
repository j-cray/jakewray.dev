use sqlx::postgres::PgPoolOptions;
use std::env;
use std::io::{self, Write};
use bcrypt::{hash, DEFAULT_COST};
use dotenvy::dotenv;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    println!("Database connection established.");

    let mut username = String::new();
    print!("Enter new admin username: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut username)?;
    let username = username.trim();

    let mut password = String::new();
    print!("Enter new admin password: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut password)?;
    let password = password.trim();

    if username.is_empty() || password.is_empty() {
        println!("Username and password cannot be empty.");
        return Ok(());
    }

    let hashed_password = hash(password, DEFAULT_COST)?;

    sqlx::query(
        "INSERT INTO users (username, password_hash) VALUES ($1, $2)",
    )
    .bind(username)
    .bind(hashed_password)
    .execute(&pool)
    .await?;

    println!("Admin user '{}' created successfully.", username);

    Ok(())
}
