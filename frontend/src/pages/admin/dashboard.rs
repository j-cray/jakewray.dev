use leptos::prelude::*;
<<<<<<< HEAD
use leptos::*;
=======
>>>>>>> origin/main

#[component]
pub fn AdminDashboard() -> impl IntoView {
    view! {
        <div class="container py-12">
            <h1 class="text-4xl mb-8">"Admin Dashboard"</h1>

            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                <a href="/admin/compose" class="card group">
                    <h3 class="text-xl font-bold mb-2 group-hover:text-brand transition-colors">"New Post"</h3>
                    <p class="text-gray-400">"Write a new blog post or article."</p>
                </a>

                <a href="/admin/sync" class="card group">
                    <h3 class="text-xl font-bold mb-2 group-hover:text-brand transition-colors">"Sync Manager"</h3>
                    <p class="text-gray-400">"Manage data sync from terracestandard.com"</p>
                </a>

                <a href="/admin/media" class="card group">
                    <h3 class="text-xl font-bold mb-2 group-hover:text-brand transition-colors">"Media Library"</h3>
                    <p class="text-gray-400">"Upload and manage photos/videos."</p>
                </a>

                 <div class="card bg-white/5 border-dashed border-white/20">
                    <h3 class="text-xl font-bold mb-2">"Stats"</h3>
                    <p class="text-gray-500 font-mono">"Total Articles: [TODO]"</p>
                </div>
            </div>
        </div>
    }
}
