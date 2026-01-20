use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct LoginResponse {
    token: String,
}

#[component]
pub fn AdminLoginPage() -> impl IntoView {
    let (username, set_username) = signal("".to_string());
    let (password, set_password) = signal("".to_string());
    let (error, set_error) = signal("".to_string());
    let (loading, set_loading) = signal(false);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_loading.set(true);
        set_error.set("".to_string());

        let username_val = username.get();
        let password_val = password.get();

        spawn_local(async move {
            let client = reqwest::Client::new();
            let req = LoginRequest {
                username: username_val,
                password: password_val,
            };

            match client
                .post("/api/admin/login")
                .json(&req)
                .send()
                .await
            {
                Ok(resp) => {
                    if resp.status().is_success() {
                        match resp.json::<LoginResponse>().await {
                            Ok(data) => {
                                // Store token in localStorage
                                let window = web_sys::window().unwrap();
                                let local_storage = window.local_storage().unwrap().unwrap();
                                let _ = local_storage.set_item("admin_token", &data.token);
                                
                                let navigate = use_navigate();
                                navigate("/admin/dashboard", Default::default());
                            }
                            Err(_) => {
                                set_error.set("Failed to parse response".to_string());
                            }
                        }
                    } else {
                        set_error.set("Invalid username or password".to_string());
                    }
                }
                Err(_) => {
                    set_error.set("Failed to connect to server".to_string());
                }
            }
            set_loading.set(false);
        });
    };

    view! {
        <div class="flex items-center justify-center min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100">
            <div class="card w-full max-w-md bg-white p-8 rounded-lg shadow-lg border border-indigo-100">
                <h1 class="text-3xl font-bold mb-2 text-center text-indigo-900">"Admin Access"</h1>
                <p class="text-center text-gray-600 mb-6">"Secure dashboard login"</p>
                
                {move || {
                    (!error.get().is_empty()).then(|| view! {
                        <div class="mb-4 p-3 bg-red-50 border border-red-200 text-red-700 rounded-md text-sm">
                            {error.get()}
                        </div>
                    })
                }}

                <form on:submit=on_submit class="flex flex-col gap-4">
                    <div class="flex flex-col gap-2">
                        <label class="text-sm font-semibold text-gray-700">"Username"</label>
                        <input
                            type="text"
                            placeholder="Enter username"
                            class="p-3 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500 transition"
                            on:input=move |ev| set_username.set(event_target_value(&ev))
                            disabled=move || loading.get()
                        />
                    </div>
                    
                    <div class="flex flex-col gap-2">
                        <label class="text-sm font-semibold text-gray-700">"Password"</label>
                        <input
                            type="password"
                            placeholder="Enter password"
                            class="p-3 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500 transition"
                            on:input=move |ev| set_password.set(event_target_value(&ev))
                            disabled=move || loading.get()
                        />
                    </div>
                    
                    <button 
                        type="submit" 
                        class="bg-indigo-600 text-white p-3 rounded-md font-bold hover:bg-indigo-700 transition disabled:opacity-50 disabled:cursor-not-allowed"
                        disabled=move || loading.get()
                    >
                        {move || if loading.get() { "Logging in..." } else { "Login" }}
                    </button>
                </form>
            </div>
        </div>
    }
}
