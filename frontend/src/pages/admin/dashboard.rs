use leptos::prelude::*;
use leptos_router::hooks::*;

#[component]
pub fn AdminDashboard() -> impl IntoView {
    let navigate = use_navigate();
    
    // Check if user is authenticated
    Effect::new(move || {
        let window = web_sys::window().unwrap();
        let local_storage = window.local_storage().unwrap().unwrap();
        let token = local_storage.get_item("admin_token").unwrap_or(None);
        
        if token.is_none() {
            navigate("/admin/login", Default::default());
        }
    });

    let logout = move |_| {
        let window = web_sys::window().unwrap();
        let local_storage = window.local_storage().unwrap().unwrap();
        let _ = local_storage.remove_item("admin_token");
        navigate("/admin/login", Default::default());
    };

    view! {
        <div class="container py-12">
            <div class="flex justify-between items-center mb-8">
                <h1 class="text-4xl font-bold text-indigo-900">"Admin Dashboard"</h1>
                <button 
                    on:click=logout 
                    class="px-4 py-2 bg-red-600 text-white rounded-md hover:bg-red-700 transition"
                >
                    "Logout"
                </button>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                <a href="/admin/compose" class="card hover:shadow-xl transition cursor-pointer hover:border-indigo-300">
                    <div class="flex items-center gap-3 mb-2">
                        <span class="text-2xl">"✏️"</span>
                        <h3 class="text-xl font-bold">"New Post"</h3>
                    </div>
                    <p class="text-gray-600">"Write a new blog post or article."</p>
                </a>

                <a href="/admin/sync" class="card hover:shadow-xl transition cursor-pointer hover:border-indigo-300">
                    <div class="flex items-center gap-3 mb-2">
                        <span class="text-2xl">"🔄"</span>
                        <h3 class="text-xl font-bold">"Sync Manager"</h3>
                    </div>
                    <p class="text-gray-600">"Manage data sync from terracestandard.com"</p>
                </a>

                <a href="/admin/media" class="card hover:shadow-xl transition cursor-pointer hover:border-indigo-300">
                    <div class="flex items-center gap-3 mb-2">
                        <span class="text-2xl">"🖼️"</span>
                        <h3 class="text-xl font-bold">"Media Library"</h3>
                    </div>
                    <p class="text-gray-600">"Upload and manage photos/videos."</p>
                </a>

                <div class="card bg-gradient-to-br from-indigo-50 to-blue-50 border-indigo-200">
                    <div class="flex items-center gap-3 mb-2">
                        <span class="text-2xl">"📊"</span>
                        <h3 class="text-xl font-bold text-indigo-900">"Stats"</h3>
                    </div>
                    <p class="text-indigo-700">"Total Articles: [TODO]"</p>
                </div>
            </div>
        </div>
    }
}
