use leptos::prelude::*;
use leptos::*;

#[component]
pub fn AdminDashboard() -> impl IntoView {
    view! {
        <div class="container py-12">
            <h1 class="text-4xl mb-8">"Admin Dashboard"</h1>

            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                <a href="/admin/compose" class="card hover:shadow-lg transition cursor-pointer">
                    <h3 class="text-xl font-bold mb-2">"New Post"</h3>
                    <p class="text-muted">"Write a new blog post or article."</p>
                </a>

                <a href="/admin/sync" class="card hover:shadow-lg transition cursor-pointer">
                    <h3 class="text-xl font-bold mb-2">"Sync Manager"</h3>
                    <p class="text-muted">"Manage data sync from terracestandard.com"</p>
                </a>

                <a href="/admin/media" class="card hover:shadow-lg transition cursor-pointer">
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
