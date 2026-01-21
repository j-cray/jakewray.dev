use leptos::prelude::*;
#[cfg(feature = "hydrate")]
use leptos_router::hooks::*;

#[component]
pub fn AdminDashboard() -> impl IntoView {
    #[cfg(feature = "hydrate")]
    let navigate = use_navigate();

    #[cfg(feature = "hydrate")]
    {
        // Check if user is authenticated
        let navigate_clone = navigate.clone();
        Effect::new(move || {
            let window = web_sys::window().unwrap();
            let local_storage = window.local_storage().unwrap().unwrap();
            let token = local_storage.get_item("admin_token").unwrap_or(None);

            if token.is_none() {
                navigate_clone("/admin/login", Default::default());
            }
        });
    }

    #[cfg(feature = "hydrate")]
    let logout = move |_| {
        let window = web_sys::window().unwrap();
        let local_storage = window.local_storage().unwrap().unwrap();
        let _ = local_storage.remove_item("admin_token");
        navigate("/admin/login", Default::default());
    };

    #[cfg(not(feature = "hydrate"))]
    let logout = move |_| {};

    view! {
        <div class="container py-12">
            <div class="flex justify-between items-center mb-8">
                <h1 class="text-4xl">"Admin Dashboard"</h1>
                <button 
                    on:click=logout 
                    class="btn btn-secondary"
                >
                    "Logout"
                </button>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                <a href="/admin/compose" class="card hover:shadow-md transition">
                    <h3 class="text-xl font-bold mb-2">"New Post"</h3>
                    <p class="text-muted">"Write a new blog post or article."</p>
                </a>

                <a href="/admin/sync" class="card hover:shadow-md transition">
                    <h3 class="text-xl font-bold mb-2">"Sync Manager"</h3>
                    <p class="text-muted">"Manage data sync from terracestandard.com"</p>
                </a>

                <a href="/admin/media" class="card hover:shadow-md transition">
                    <h3 class="text-xl font-bold mb-2">"Media Library"</h3>
                    <p class="text-muted">"Upload and manage photos/videos."</p>
                </a>

                <div class="card">
                    <h3 class="text-xl font-bold mb-2">"Stats"</h3>
                    <p class="text-muted">"Total Articles: [TODO]"</p>
                </div>
            </div>
        </div>
    }
}
