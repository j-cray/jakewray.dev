use crate::context::use_admin_context;
use leptos::prelude::*;

#[component]
pub fn PersonalBlogPage() -> impl IntoView {
    let admin_ctx = use_admin_context();

    view! {
        <div class="container py-12">
            <div class="flex justify-between items-center mb-6">
                <h1 class="text-4xl">"Blog"</h1>
                {move || {
                    if admin_ctx.is_admin.get() {
                        view! {
                            <a href="/admin/compose" class="btn btn-primary">
                                "Compose New Post"
                            </a>
                        }
                        .into_any()
                    } else {
                        view! { <span class="hidden" /> }.into_any()
                    }
                }}
            </div>
            <p class="text-muted">"Coming soon."</p>
        </div>
    }
}
