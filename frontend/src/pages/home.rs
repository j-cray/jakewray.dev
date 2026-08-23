use crate::api::articles::get_articles;
use crate::api::blog::{get_recent_blog_posts, BlogPostItem};
use crate::pages::journalism::card::render_article_card;
use crate::pages::programming::components::RepoCard;
use crate::pages::programming::types::get_pinned_repos;
use crate::utils::sorting::{extract_highlight_articles, HIGHLIGHT_SLUGS};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

#[component]
pub fn HomeBlogPostCard(post: BlogPostItem) -> impl IntoView {
    let post_url = format!("/blog#{}", post.slug);
    let title = post.title.clone();
    let excerpt = post.excerpt.clone();
    let date = post.display_date.clone();
    let tags = post.tags.clone();

    view! {
        <article class="card blog-card">
            <div class="blog-card-meta">
                <span class="blog-card-date">{date}</span>
            </div>
            <h3 class="blog-card-title">
                <a href=post_url.clone() class="blog-card-link">
                    {title}
                </a>
            </h3>
            <p class="blog-card-excerpt">{excerpt}</p>
            {if !tags.is_empty() {
                view! {
                    <div class="blog-card-tags">
                        {tags.into_iter().map(|tag| {
                            view! { <span class="tag-chip">{tag}</span> }
                        }).collect_view()}
                    </div>
                }.into_any()
            } else {
                view! { <span class="hidden" /> }.into_any()
            }}
        </article>
    }
}

#[component]
pub fn BlogComingSoonCard() -> impl IntoView {
    view! {
        <div class="card coming-soon-card">
            <div class="coming-soon-badge">
                <svg xmlns="http://www.w3.org/2000/svg" class="icon-sm" viewBox="0 0 20 20" fill="currentColor">
                    <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm1-12a1 1 0 10-2 0v4a1 1 0 00.293.707l2.828 2.829a1 1 0 101.415-1.415L11 9.586V6z" clip-rule="evenodd" />
                </svg>
                <span>"In the works"</span>
            </div>
            <h3 class="coming-soon-title">"Coming soon"</h3>
            <p class="coming-soon-text">
                "Blog posts are currently in development. Check back soon for new articles and technical write-ups."
            </p>
        </div>
    }
}

#[component]
pub fn HomePage() -> impl IntoView {
    let pinned_repos = get_pinned_repos();
    let blog_posts_resource = Resource::new(|| (), |_| get_recent_blog_posts(Some(6)));
    let articles_resource = Resource::new(|| (), |_| get_articles());

    view! {
        <div class="container home-hero">
            <header class="hero">
                <h1 class="hero-title">"Jake Wray"</h1>
                <p class="hero-subtitle">
                    "A work in progress (me and the website)"
                </p>
            </header>

            <div class="home-columns">
                <section class="home-column">
                    <div class="home-column-header">
                        <h2 class="home-column-title">"Pinned repositories"</h2>
                        <a href="/code" class="home-column-link">"View all →"</a>
                    </div>
                    <div class="home-column-cards">
                        {pinned_repos.into_iter().map(|repo| {
                            view! { <RepoCard repo=repo /> }
                        }).collect_view()}
                    </div>
                </section>

                <section class="home-column">
                    <div class="home-column-header">
                        <h2 class="home-column-title">"Latest posts"</h2>
                        <a href="/blog" class="home-column-link">"View blog →"</a>
                    </div>
                    <div class="home-column-cards">
                        <Suspense fallback=move || view! { <p class="text-muted">"Loading posts..."</p> }>
                            {move || {
                                blog_posts_resource.get().map(|res| {
                                    match res {
                                        Ok(posts) if !posts.is_empty() => {
                                            view! {
                                                <div class="home-posts-list">
                                                    {posts.into_iter().map(|post| {
                                                        view! { <HomeBlogPostCard post=post /> }
                                                    }).collect_view()}
                                                </div>
                                            }.into_any()
                                        }
                                        _ => {
                                            view! {
                                                <BlogComingSoonCard />
                                            }.into_any()
                                        }
                                    }
                                })
                            }}
                        </Suspense>
                    </div>
                </section>

                <section class="home-column">
                    <div class="home-column-header">
                        <h2 class="home-column-title">"Top articles"</h2>
                        <a href="/journalism" class="home-column-link">"View all →"</a>
                    </div>
                    <div class="home-column-cards">
                        <Suspense fallback=move || view! { <p class="text-muted">"Loading articles..."</p> }>
                            {move || {
                                articles_resource.get().map(|res| {
                                    match res {
                                        Ok(articles) => {
                                            let highlight_articles = extract_highlight_articles(&articles, HIGHLIGHT_SLUGS);
                                            if highlight_articles.is_empty() {
                                                view! {
                                                    <div class="card coming-soon-card">
                                                        <h3 class="coming-soon-title">"Articles"</h3>
                                                        <p class="coming-soon-text">"No highlighted articles available."</p>
                                                    </div>
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <div class="home-articles-list">
                                                        {highlight_articles.iter().map(render_article_card).collect_view()}
                                                    </div>
                                                }.into_any()
                                            }
                                        }
                                        Err(e) => {
                                            view! {
                                                <p class="text-red-500">"Error loading articles: " {e.to_string()}</p>
                                            }.into_any()
                                        }
                                    }
                                })
                            }}
                        </Suspense>
                    </div>
                </section>
            </div>
        </div>
    }
}

#[component]
pub fn NotFound() -> impl IntoView {
    view! {
        <div class="container py-24 text-center">
            <h1 class="text-4xl mb-4">"404"</h1>
            <p>"Page not found."</p>
        </div>
    }
}

#[component]
pub fn AdminRedirect() -> impl IntoView {
    let navigate = use_navigate();
    leptos::prelude::Effect::new(move || {
        navigate("/admin/login", Default::default());
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pinned_repos_for_home_page() {
        let repos = get_pinned_repos();
        assert_eq!(repos.len(), 5);
        assert_eq!(repos[0].name, "jakewray.dev");
    }

    #[test]
    fn test_home_blog_post_card_content() {
        let post = BlogPostItem {
            id: "test-id".to_string(),
            slug: "test-slug".to_string(),
            title: "Test Blog Post".to_string(),
            excerpt: "Excerpt of test post".to_string(),
            display_date: "August 23, 2026".to_string(),
            iso_date: "2026-08-23".to_string(),
            tags: vec!["rust".to_string()],
        };

        assert_eq!(post.slug, "test-slug");
        assert_eq!(post.title, "Test Blog Post");
        assert_eq!(post.tags, vec!["rust"]);
    }
}
