use leptos::prelude::*;
<<<<<<< HEAD
use shared::{Article, BlogPost};

#[server]
pub async fn get_articles() -> Result<Vec<Article>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use sqlx::{PgPool, Row};
        let pool = use_context::<PgPool>().ok_or(ServerFnError::new("Pool not found"))?;

        let articles = sqlx::query("SELECT id, wp_id, slug, title, subtitle, excerpt, content, cover_image_url, author, published_at, origin FROM articles ORDER BY published_at DESC LIMIT 20")
            .map(|row: sqlx::postgres::PgRow| Article {
                id: row.get("id"),
                wp_id: row.get("wp_id"),
                slug: row.get("slug"),
                title: row.get("title"),
                subtitle: row.get("subtitle"),
                excerpt: row.get("excerpt"),
                content: row.get("content"),
                cover_image_url: row.get("cover_image_url"),
                author: row.get("author"),
                published_at: row.get("published_at"),
                origin: row.get("origin"),
            })
            .fetch_all(&pool)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        Ok(articles)
    }
    #[cfg(not(feature = "ssr"))]
    {
        unreachable!()
    }
}

#[server]
pub async fn get_blog_posts() -> Result<Vec<BlogPost>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use sqlx::{PgPool, Row};
        let pool = use_context::<PgPool>().ok_or(ServerFnError::new("Pool not found"))?;

        let posts = sqlx::query("SELECT id, slug, title, content, published_at, tags FROM blog_posts ORDER BY published_at DESC LIMIT 20")
            .map(|row: sqlx::postgres::PgRow| BlogPost {
                id: row.get("id"),
                slug: row.get("slug"),
                title: row.get("title"),
                content: row.get("content"),
                published_at: row.get("published_at"),
                tags: row.get("tags"),
            })
            .fetch_all(&pool)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        Ok(posts)
    }
    #[cfg(not(feature = "ssr"))]
    {
        unreachable!()
    }
}
=======
>>>>>>> origin/main

#[component]
pub fn JournalismPage() -> impl IntoView {
    let articles = Resource::new(|| (), |_| async move {
        get_articles().await.unwrap_or_default()
    });

    view! {
        <div class="container mx-auto py-12 px-4">
            <h1 class="text-4xl mb-8 font-bold text-white">"Journalism"</h1>
             <div class="flex gap-4 mb-8 overflow-x-auto pb-2">
                <button class="px-4 py-2 rounded-full border border-gray-600 hover:bg-gray-800 text-gray-300 transition-colors">"All"</button>
                <button class="px-4 py-2 rounded-full border border-gray-600 hover:bg-gray-800 text-gray-300 transition-colors">"Articles"</button>
                <button class="px-4 py-2 rounded-full border border-gray-600 hover:bg-gray-800 text-gray-300 transition-colors">"Photojournalism"</button>
                <button class="px-4 py-2 rounded-full border border-gray-600 hover:bg-gray-800 text-gray-300 transition-colors">"Video"</button>
            </div>

            <Suspense fallback=move || view! { <p class="text-gray-500">"Loading articles..."</p> }>
                {move || {
                    articles.get().map(|data| {
                        view! {
                            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                                {data.into_iter().map(|article| view! {
                                    <div class="glass p-6 rounded-xl hover:bg-white/5 transition duration-300 border border-white/10 flex flex-col h-full">
                                        <h3 class="text-xl font-bold mb-2 text-brand">{article.title}</h3>
                                        <p class="text-sm text-gray-400 mb-4">{article.published_at.format("%B %d, %Y").to_string()}</p>
                                        <p class="text-gray-300 flex-grow mb-4 line-clamp-3">{article.excerpt.unwrap_or_default()}</p>
                                        <a href=format!("/article/{}", article.slug) class="text-brand-dim mt-auto hover:text-brand font-medium inline-flex items-center gap-1">
                                            "Read More"
                                        </a>
                                    </div>
                                }).collect_view()}
                            </div>
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}

#[component]
pub fn PersonalPage() -> impl IntoView {
    let posts = Resource::new(|| (), |_| async move {
        get_blog_posts().await.unwrap_or_default()
    });

    view! {
        <div class="container mx-auto py-12 px-4">
            <h1 class="text-4xl mb-8 font-bold text-white">"Personal Blog"</h1>
            <p class="text-muted mb-8">"Thoughts, updates, and photography."</p>

             <Suspense fallback=move || view! { <p class="text-gray-500">"Loading posts..."</p> }>
                {move || {
                    posts.get().map(|data| {
                        view! {
                            <div class="grid grid-cols-1 md:grid-cols-2 gap-8">
                                {data.into_iter().map(|post| view! {
                                    <div class="glass p-8 rounded-xl hover:bg-white/5 transition duration-300 border border-white/10">
                                        <div class="flex items-center justify-between mb-4">
                                            <span class="text-xs font-bold uppercase tracking-wider text-brand-dim">
                                                {post.tags.clone().unwrap_or_default().get(0).cloned().unwrap_or("Blog".to_string())}
                                            </span>
                                            <span class="text-xs text-gray-500">{post.published_at.format("%B %d, %Y").to_string()}</span>
                                        </div>
                                        <h2 class="text-2xl font-bold mb-4 text-white hover:text-brand transition-colors">
                                            <a href=format!("/blog/{}", post.slug)>{post.title}</a>
                                        </h2>
                                        <p class="text-gray-400 line-clamp-3 mb-6">
                                            // Simple content preview - in reality might need to strip HTML or fetch excerpt if available
                                            {post.content.chars().take(150).collect::<String>()} "..."
                                        </p>
                                        <div class="flex items-center gap-2">
                                            {post.tags.clone().unwrap_or_default().iter().skip(1).map(|tag| view! {
                                                <span class="px-2 py-1 text-xs rounded-md bg-white/5 text-gray-400">{tag.clone()}</span>
                                            }).collect_view()}
                                        </div>
                                    </div>
                                }).collect_view()}
                            </div>
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}

#[component]
pub fn CreativeWritingPage() -> impl IntoView {
    view! {
        <div class="container py-12">
            <h1 class="text-4xl mb-6">"Creative Writing"</h1>
             <p class="text-muted">"Stories, Novels, and Poetry."</p>
        </div>
    }
}

#[component]
pub fn MusicPage() -> impl IntoView {
    view! {
        <div class="container py-12">
            <h1 class="text-4xl mb-6">"Music"</h1>
             <p class="text-muted">"Original compositions."</p>
        </div>
    }
}

#[component]
pub fn VisualArtPage() -> impl IntoView {
    view! {
        <div class="container py-12">
            <h1 class="text-4xl mb-6">"Visual Art"</h1>
             <p class="text-muted">"Drawings and Digital Art."</p>
        </div>
    }
}

#[component]
pub fn ProgrammingPage() -> impl IntoView {
    view! {
        <div class="container py-12">
             <h1 class="text-4xl mb-6">"Programming"</h1>
             <p class="text-muted">"GitHub Showcase."</p>
        </div>
    }
}
