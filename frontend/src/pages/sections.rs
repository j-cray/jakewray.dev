use crate::data::journalism;
use leptos::either::Either;
use leptos::prelude::*;
use leptos::prelude::IntoAny;
use leptos_router::hooks::use_params_map;

#[component]
pub fn JournalismPage() -> impl IntoView {
    let articles = journalism::all_articles();

    view! {
        <div class="container py-12">
            <h1 class="text-4xl mb-4">"Journalism"</h1>
            <p class="text-gray-700 mb-10 max-w-3xl">
                "Reporting on northern communities, Indigenous culture, and public interest stories."
            </p>

            <div class="journalism-grid">
                {articles
                    .iter()
                    .map(|article| {
                        let slug = article.slug.clone();
                        let title = article.title.clone();
                        let excerpt = article.excerpt.clone();
                        let date = article.display_date.clone();
                        let image = article.images.get(0).cloned();
                        let thumb = image
                            .as_ref()
                            .map(|src| view! { <img src=src class="journalism-img" alt="article thumbnail"/> }.into_any())
                            .unwrap_or_else(|| {
                                view! { <div class="journalism-placeholder">"Image coming soon"</div> }.into_any()
                            });
                        view! {
                            <a href=format!("/journalism/{}", slug) class="journalism-card">
                                <div class="journalism-thumb">
                                    {thumb}
                                </div>
                                <div class="journalism-body">
                                    <p class="journalism-date">{date}</p>
                                    <h3 class="journalism-title">{title}</h3>
                                    <p class="journalism-excerpt">{excerpt}</p>
                                    <div class="journalism-link">"Read more →"</div>
                                </div>
                            </a>
                        }
                    })
                    .collect_view()}
            </div>
        </div>
    }
}

#[component]
pub fn JournalismArticlePage() -> impl IntoView {
    let params = use_params_map();
    let slug = move || params.with(|p| p.get("slug").map(|s| s.to_string()).unwrap_or_default());
    let article = move || journalism::find_article(&slug());

    view! {
        <div class="container py-12 max-w-4xl">
            {move || {
                match article() {
                    Some(article) => {
                        Either::Left({
                            let display_date = article.display_date.clone();
                            let title = article.title.clone();
                            let source_url = article.source_url.clone();
                            let images = article.images.clone();
                            let content_html = article.content_html.clone();
                            view! {
                                <>
                                    <p class="text-sm text-gray-500 mb-2">{display_date}</p>
                                    <h1 class="mb-4 text-4xl font-bold text-gray-900">{title}</h1>
                                    <div class="mb-6 flex flex-wrap items-center gap-3 text-sm text-gray-600">
                                        <a class="underline" href="/journalism">"Back to journalism"</a>
                                        <span class="text-gray-400">"•"</span>
                                        <a class="underline" href=source_url target="_blank" rel="noreferrer">
                                            "Original publication"
                                        </a>
                                    </div>
                                    {(!images.is_empty()).then(|| {
                                        view! {
                                            <div class="mb-8 flex flex-wrap gap-3">
                                                {images
                                                    .iter()
                                                    .map(|src| view! { <img src=src class="h-32 w-auto rounded" alt="article image"/> })
                                                    .collect_view()}
                                            </div>
                                        }
                                    })}
                                    <div class="article-content prose max-w-none" inner_html=content_html></div>
                                </>
                            }
                        })
                    }
                    None => Either::Right(view! { 
                        <p>"Article not found."</p>
                    }),
                }
            }}
        </div>
    }
}

#[component]
pub fn PersonalPage() -> impl IntoView {
    view! {
        <div class="container py-12">
            <h1 class="text-4xl mb-6">"Personal"</h1>
            <p class="text-gray-600 mb-8">"Blog, Creative Writing, Photography, and Videography."</p>

            <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                <a href="/personal/blog" class="card hover:shadow-lg transition-shadow">
                    <h3 class="text-xl font-bold mb-2">"Blog"</h3>
                    <p class="text-muted">"Personal thoughts and musings"</p>
                </a>

                <a href="/personal/writing" class="card hover:shadow-lg transition-shadow">
                    <h3 class="text-xl font-bold mb-2">"Creative Writing"</h3>
                    <p class="text-muted">"Stories, novels, and poetry"</p>
                </a>

                <div class="card opacity-50">
                    <h3 class="text-xl font-bold mb-2">"Photography"</h3>
                    <p class="text-muted">"Coming soon"</p>
                </div>

                <div class="card opacity-50">
                    <h3 class="text-xl font-bold mb-2">"Videography"</h3>
                    <p class="text-muted">"Coming soon"</p>
                </div>
            </div>
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

#[component]
pub fn PersonalBlogPage() -> impl IntoView {
    view! {
        <div class="container py-12">
            <h1 class="text-4xl mb-6">"Blog"</h1>
            <p class="text-muted">"Personal thoughts and musings."</p>
        </div>
    }
}
