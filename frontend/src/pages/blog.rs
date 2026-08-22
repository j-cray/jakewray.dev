use leptos::prelude::*;

#[component]
pub fn PersonalBlogPage() -> impl IntoView {
    let (is_admin, _set_is_admin) = signal(false);

    Effect::new(move || {
        #[cfg(target_arch = "wasm32")]
        {
            if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
                if let Ok(Some(t)) = storage.get_item("admin_token") {
                    if !t.is_empty() && !shared::auth::is_token_expired(&t) {
                        _set_is_admin.set(true);
                    }
                }
            }
        }
    });

    view! {
        <div class="container py-12">
            <div class="flex justify-between items-center mb-6">
                <h1 class="text-4xl">"Blog"</h1>
                {move || {
                    if is_admin.get() {
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
            <p class="text-muted">"Personal thoughts and musings."</p>
        </div>
    }
}
