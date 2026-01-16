use dotenvy::dotenv;
use serde::Deserialize;
use sqlx::postgres::PgPoolOptions;
use std::env;

#[derive(Debug, Deserialize)]
struct WpPost {
    id: i64,
    date_gmt: String,
    slug: String,
    title: WpContent,
    content: WpContent,
    excerpt: WpContent,
    _embedded: Option<Embedded>,
}

#[derive(Debug, Deserialize)]
struct WpContent {
    rendered: String,
}

#[derive(Debug, Deserialize)]
struct Embedded {
    author: Option<Vec<WpAuthor>>,
}

#[derive(Debug, Deserialize)]
struct WpAuthor {
    name: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new().connect(&database_url).await?;

    println!("Syncing from terracestandard.com...");
    sync_terracestandard(&pool).await?;

    Ok(())
}

async fn sync_terracestandard(pool: &sqlx::PgPool) -> Result<(), Box<dyn std::error::Error>> {
    // Need _embed to get author info
    let url = "https://terracestandard.com/wp-json/wp/v2/posts?per_page=20&_embed";
    let client = reqwest::Client::new();
    let resp = client.get(url).send().await?;

    if !resp.status().is_success() {
        println!("Failed to fetch posts: {}", resp.status());
        return Ok(());
    }

    let posts: Vec<WpPost> = resp.json().await?;
    println!("Found {} posts (checking authors...)", posts.len());

    for post in posts {
        // Filter by author
        let is_me = if let Some(embedded) = &post._embedded {
            if let Some(authors) = &embedded.author {
                authors.iter().any(|a| a.name.contains("Jake Wray")) // Flexible match
            } else {
                false
            }
        } else {
            false
        };

        if !is_me {
            continue;
        }

        let title = post.title.rendered;
        let content = post.content.rendered;
        let slug = post.slug;
        let wp_id = post.id;
        let published_at = chrono::DateTime::parse_from_rfc3339(&format!("{}Z", post.date_gmt))
            .unwrap_or_else(|_| chrono::Utc::now().into())
            .with_timezone(&chrono::Utc);

        // Upsert
        println!("Syncing: {}", title);
        sqlx::query(
            r#"
            INSERT INTO articles (wp_id, slug, title, content, published_at, author, origin)
            VALUES ($1, $2, $3, $4, $5, 'Jake Wray', 'synced')
            ON CONFLICT (wp_id) DO UPDATE SET
                title = EXCLUDED.title,
                content = EXCLUDED.content,
                updated_at = NOW()
            "#,
        )
        .bind(wp_id)
        .bind(slug)
        .bind(title)
        .bind(content)
        .bind(published_at)
        .execute(pool)
        .await?;
    }

    Ok(())
}
