use crate::api::articles::{delete_article, get_drafts_and_scheduled, save_article, Article};
use leptos::prelude::*;
use leptos::task::spawn_local;
#[cfg(target_arch = "wasm32")]
use leptos_router::hooks::*;

#[component]
pub fn AdminDashboard() -> impl IntoView {
    #[cfg(target_arch = "wasm32")]
    let navigate = use_navigate();
    let admin_ctx = crate::context::use_admin_context();
    let (refresh_counter, set_refresh_counter) = signal(0);
    let (action_message, set_action_message) = signal(String::new());

    #[cfg(target_arch = "wasm32")]
    let nav_auth = navigate.clone();
    Effect::new(move || {
        #[cfg(target_arch = "wasm32")]
        {
            if !admin_ctx.is_admin.get() {
                nav_auth("/admin/login", Default::default());
            }
        }
    });

    let drafts_resource = Resource::new(
        move || (admin_ctx.token.get(), refresh_counter.get()),
        |(t, _)| async move {
            if t.is_empty() {
                Ok(Vec::<Article>::new())
            } else {
                get_drafts_and_scheduled(t).await
            }
        },
    );

    #[cfg(target_arch = "wasm32")]
    let logout = move |_| {
        admin_ctx.logout();
        navigate("/admin/login", Default::default());
    };

    #[cfg(not(target_arch = "wasm32"))]
    let logout = move |_| {};

    let publish_now = move |mut article: Article| {
        let tok = admin_ctx.token.get();
        if tok.is_empty() {
            return;
        }
        set_action_message.set("Publishing...".to_string());
        article.status = Some("published".to_string());
        spawn_local(async move {
            match save_article(tok, article).await {
                Ok(_) => {
                    set_action_message.set("Post published successfully!".to_string());
                    set_refresh_counter.update(|c| *c += 1);
                }
                Err(e) => {
                    set_action_message.set(format!("Error publishing post: {}", e));
                }
            }
        });
    };

    let delete_draft = move |slug: String| {
        let tok = admin_ctx.token.get();
        if tok.is_empty() {
            return;
        }
        set_action_message.set("Deleting...".to_string());
        spawn_local(async move {
            match delete_article(tok, slug).await {
                Ok(_) => {
                    set_action_message.set("Post deleted successfully!".to_string());
                    set_refresh_counter.update(|c| *c += 1);
                }
                Err(e) => {
                    set_action_message.set(format!("Error deleting post: {}", e));
                }
            }
        });
    };

    view! {
        <div class="container py-12">
            <div class="flex justify-between items-center mb-8">
                <h1 class="text-4xl font-bold">"Admin Dashboard"</h1>
                <div class="flex gap-2">
                    <a href="/admin/password-change" class="btn btn-secondary">
                        "Change Password"
                    </a>
                    <button
                        on:click=logout
                        class="btn btn-secondary text-red-600 hover:text-red-700"
                    >
                        "Logout"
                    </button>
                </div>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 mb-12">
                <a href="/admin/compose" class="card hover:shadow-md transition bg-white border p-6 rounded-xl">
                    <div class="flex items-center gap-3 mb-2">
                        <div class="p-2 bg-sky-50 text-sky-600 rounded-lg">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                            </svg>
                        </div>
                        <h3 class="text-xl font-bold">"New Post"</h3>
                    </div>
                    <p class="text-gray-600 text-sm">"Write a new blog post or article with rich text formatting."</p>
                </a>

                <a href="/admin/media" class="card hover:shadow-md transition bg-white border p-6 rounded-xl">
                    <div class="flex items-center gap-3 mb-2">
                        <div class="p-2 bg-purple-50 text-purple-600 rounded-lg">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
                            </svg>
                        </div>
                        <h3 class="text-xl font-bold">"Media Library"</h3>
                    </div>
                    <p class="text-gray-600 text-sm">"Upload and manage journalism photos and media."</p>
                </a>

                <a href="/about" class="card hover:shadow-md transition bg-white border p-6 rounded-xl">
                    <div class="flex items-center gap-3 mb-2">
                        <div class="p-2 bg-emerald-50 text-emerald-600 rounded-lg">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
                            </svg>
                        </div>
                        <h3 class="text-xl font-bold">"About Me Page"</h3>
                    </div>
                    <p class="text-gray-600 text-sm">"Edit and manage the biography content on your About Me page."</p>
                </a>
            </div>

            // Drafts & Scheduled Posts Section
            <div class="bg-white border rounded-xl p-6 shadow-sm mb-8">
                <div class="flex items-center justify-between mb-6 pb-4 border-b">
                    <div>
                        <h2 class="text-2xl font-bold text-gray-900 flex items-center gap-2">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6 text-amber-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                            </svg>
                            "Drafts & Scheduled Posts"
                        </h2>
                        <p class="text-sm text-gray-500">"Posts that are saved as drafts or queued for future publication."</p>
                    </div>
                    {move || {
                        let msg = action_message.get();
                        if !msg.is_empty() {
                            view! { <span class="text-sm text-sky-700 bg-sky-50 px-3 py-1.5 rounded font-medium">{msg}</span> }.into_any()
                        } else {
                            view! { <span class="hidden" /> }.into_any()
                        }
                    }}
                </div>

                <Suspense fallback=move || view! { <p class="text-gray-500 py-4 text-center">"Loading drafts..."</p> }>
                    {move || {
                        let drafts_res = drafts_resource.get();
                        match drafts_res {
                            Some(Ok(items)) if !items.is_empty() => {
                                view! {
                                    <div class="divide-y border rounded-lg overflow-hidden">
                                        {items.into_iter().map(|item| {
                                            let item_clone = item.clone();
                                            let slug_for_edit = item.slug.clone();
                                            let slug_for_delete = item.slug.clone();
                                            let status_str = item.status.clone().unwrap_or_else(|| "draft".to_string());
                                            let is_scheduled = status_str == "scheduled";

                                            view! {
                                                <div class="p-4 flex flex-col md:flex-row md:items-center justify-between gap-4 hover:bg-gray-50/80 transition-colors">
                                                    <div class="flex-grow">
                                                        <div class="flex items-center gap-3 mb-1">
                                                            {if is_scheduled {
                                                                view! {
                                                                    <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-semibold bg-sky-100 text-sky-800 border border-sky-200">
                                                                        "Scheduled"
                                                                    </span>
                                                                }.into_any()
                                                            } else {
                                                                view! {
                                                                    <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-semibold bg-amber-100 text-amber-800 border border-amber-200">
                                                                        "Draft"
                                                                    </span>
                                                                }.into_any()
                                                            }}
                                                            <h4 class="text-lg font-bold text-gray-900">
                                                                {if item.title.trim().is_empty() { "(Untitled Post)".to_string() } else { item.title }}
                                                            </h4>
                                                        </div>
                                                        <div class="text-xs text-gray-500 flex items-center gap-4">
                                                            <span>"Slug: " <code class="bg-gray-100 px-1 py-0.5 rounded">{item.slug}</code></span>
                                                            {if is_scheduled {
                                                                view! { <span>"Publish date: " <strong>{item.display_date}</strong></span> }.into_any()
                                                            } else {
                                                                view! { <span class="hidden" /> }.into_any()
                                                            }}
                                                        </div>
                                                    </div>

                                                    <div class="flex items-center gap-2 self-end md:self-center">
                                                        <a
                                                            href=format!("/admin/compose?slug={}", slug_for_edit)
                                                            class="btn btn-sm btn-secondary flex items-center gap-1 text-gray-700"
                                                        >
                                                            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                                                            </svg>
                                                            "Edit"
                                                        </a>

                                                        <button
                                                            type="button"
                                                            class="btn btn-sm btn-secondary text-emerald-700 hover:text-emerald-800 flex items-center gap-1"
                                                            on:click=move |_| publish_now(item_clone.clone())
                                                        >
                                                            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                                                            </svg>
                                                            "Publish Now"
                                                        </button>

                                                        <button
                                                            type="button"
                                                            class="btn btn-sm btn-secondary text-red-600 hover:text-red-700 flex items-center gap-1"
                                                            on:click=move |_| delete_draft(slug_for_delete.clone())
                                                        >
                                                            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                                                            </svg>
                                                            "Delete"
                                                        </button>
                                                    </div>
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_any()
                            }
                            Some(Ok(_)) => {
                                view! {
                                    <div class="text-center py-8 text-gray-500 bg-gray-50 rounded-lg border border-dashed">
                                        <p class="font-medium">"No drafts or scheduled posts right now."</p>
                                        <a href="/admin/compose" class="text-sky-600 font-semibold hover:underline text-sm mt-1 inline-block">
                                            "Compose a new post →"
                                        </a>
                                    </div>
                                }.into_any()
                            }
                            Some(Err(e)) => {
                                view! { <p class="text-red-600 py-4 text-center">{format!("Error loading drafts: {}", e)}</p> }.into_any()
                            }
                            None => view! { <p class="text-gray-500 py-4 text-center">"Loading..."</p> }.into_any(),
                        }
                    }}
                </Suspense>
            </div>
        </div>
    }
}
