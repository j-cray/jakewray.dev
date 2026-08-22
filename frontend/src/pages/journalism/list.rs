use crate::api::articles::get_articles;
use crate::pages::journalism::card::render_article_card;
use crate::utils::sorting::{
    extract_highlight_articles, sort_articles_newest_first, HIGHLIGHT_SLUGS,
};
use leptos::prelude::*;

#[component]
pub fn JournalismPage() -> impl IntoView {
    let articles_resource = Resource::new(|| (), |_| get_articles());

    view! {
        <div class="container py-12">
            <Suspense fallback=move || view! { <p>"Loading articles..."</p> }>
                {move || {
                    articles_resource.get().map(|res| {
                        match res {
                            Ok(mut articles) => {
                                let highlight_articles = extract_highlight_articles(&articles, HIGHLIGHT_SLUGS);
                                sort_articles_newest_first(&mut articles);

                                view! {
                                    <div>
                                        <section class="mb-12">
                                            <h2 class="text-3xl font-bold mb-6">"Journalism Highlights"</h2>
                                            <div class="journalism-grid">
                                                {highlight_articles.iter().map(render_article_card).collect_view()}
                                            </div>
                                        </section>

                                        <section>
                                            <h2 class="text-3xl font-bold mb-4">"All Articles"</h2>
                                            <p class="text-gray-700 mb-8 max-w-3xl">
                                                "A collection of community news articles I have written, mostly for The Terrace Standard, but some articles are from my 2017 internship at The Spruce Grove Examiner and some are even older, from my years studying journalism at Langara College."
                                            </p>
                                            <div class="journalism-grid">
                                                {articles.iter().map(render_article_card).collect_view()}
                                            </div>
                                        </section>
                                    </div>
                                }.into_any()
                            }
                            Err(e) => view! { <p class="text-red-500">"Error loading articles: " {e.to_string()}</p> }.into_any()
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}
