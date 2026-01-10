use sqlx::postgres::PgPoolOptions;
use serde::Deserialize;
use std::env;
use dotenvy::dotenv;
use slug::slugify;

#[derive(Debug, Deserialize)]
struct WpPost {
    id: i64,
    date_gmt: String,
    slug: String,
    title: WpContent,
    content: WpContent,
    excerpt: WpContent,
}

#[derive(Debug, Deserialize)]
struct WpContent {
    rendered: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .connect(&database_url)
        .await?;

    println!("Importing from jakewray.ca...");
    import_jakewray(&pool).await?;

    Ok(())
}

async fn import_jakewray(pool: &sqlx::PgPool) -> Result<(), Box<dyn std::error::Error>> {
    // Basic pagination loop could be added, getting 100 for now
    let url = "https://jakewray.ca/wp-json/wp/v2/posts?per_page=100";
    let resp = reqwest::get(url).await?;

    if !resp.status().is_success() {
        println!("Failed to fetch posts: {}", resp.status());
        return Ok(());
    }

    let posts: Vec<WpPost> = resp.json().await?;
    println!("Found {} posts.", posts.len());

    for post in posts {
        let title = post.title.rendered;
        let content = post.content.rendered; // keeping HTML
        let slug = post.slug;
        let published_at = chrono::DateTime::parse_from_rfc3339(&format!("{}Z", post.date_gmt))
            .unwrap_or_else(|_| chrono::Utc::now().into())
            .with_timezone(&chrono::Utc);

        // Check if exists
        let exists = sqlx::query!("SELECT id FROM blog_posts WHERE slug = $1", slug)
            .fetch_optional(pool)
            .await?;

        if exists.is_some() {
            println!("Skipping existing: {}", title);
            continue;
        }

        println!("Inserting: {}", title);
        sqlx::query!(
            "INSERT INTO blog_posts (slug, title, content, published_at) VALUES ($1, $2, $3, $4)",
            slug,
            title,
            content,
            published_at
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}
