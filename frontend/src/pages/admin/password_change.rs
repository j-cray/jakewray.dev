use leptos::prelude::*;
#[cfg(target_arch = "wasm32")]
use leptos::task::spawn_local;
#[cfg(target_arch = "wasm32")]
use leptos_router::hooks::use_navigate;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
struct ChangePasswordRequest {
    current_password: String,
    new_password: String,
}

#[component]
pub fn AdminPasswordChange() -> impl IntoView {
    let (current_password, set_current_password) = signal("".to_string());
    let (new_password, set_new_password) = signal("".to_string());
    let (confirm_password, set_confirm_password) = signal("".to_string());
    let (error, set_error) = signal("".to_string());
    let (success, set_success) = signal("".to_string());
    let (loading, set_loading) = signal(false);

    #[cfg(target_arch = "wasm32")]
    let on_submit = {
        let navigate = use_navigate();
        move |ev: leptos::ev::SubmitEvent| {
            ev.prevent_default();
            set_loading.set(true);
            set_error.set("".to_string());
            set_success.set("".to_string());

            let current = current_password.get();
            let new = new_password.get();
            let confirm = confirm_password.get();

            if new != confirm {
                set_error.set("New passwords do not match".to_string());
                set_loading.set(false);
                return;
            }

            if new.len() < 8 {
                set_error.set("Password must be at least 8 characters long".to_string());
                set_loading.set(false);
                return;
            }

            let navigate = navigate.clone();
            spawn_local(async move {
                let window = web_sys::window().unwrap();
                let local_storage = window.local_storage().unwrap().unwrap();
                let token = local_storage.get_item("admin_token").unwrap_or(None);

                if token.is_none() {
                    navigate("/admin/login", Default::default());
                    return;
                }
                let token = token.unwrap();

                let req = ChangePasswordRequest {
                    current_password: current,
                    new_password: new,
                };

                let resp = gloo_net::http::Request::post("/admin/password")
                    // The backend router is nested under `/`. `api::router` maps `/password`.
                    // But in `main.rs`: `.nest("/", api::router(app_state.clone()))`
                    // So it is just `/password`.
                    // Actually, checking `admin.rs`, it is mapped at `/password`.
                    // However, I should check if there is a global prefix.
                    // `main.rs`: `.nest("/", api::router(app_state.clone()))`
                    // So it is `/password`.
                    // Wait, `admin.rs` router returns:
                    // Router::new()
                    //     .route("/login", post(login))
                    //     .route("/password", post(change_password))
                    //     .route("/me", get(me))
                    // And `main.rs` nests it at `/`.
                    // So it is `/password`.
                    // BUT `login` was called as `/admin/login` in the frontend `login.rs`.
                    // Let me check `login.rs` again.
                    // `Request::post("/admin/login")` in `login.rs`.
                    // But `admin.rs` defines `/login`.
                    // This implies `main.rs` nests it under `/admin` or `admin.rs` defines paths starting with `/admin`?
                    // No, `admin.rs` defines `/login`.
                    // Let's re-read `main.rs`.
                    // `.nest("/", api::router(app_state.clone()))`
                    // This is suspicious. If `login.rs` calls `/admin/login`, then `api::router` must be nested under `/admin` or define route as `/admin/login`.
                    // In `admin.rs`: `.route("/login", post(login))`.
                    // This means passing `/login` to `Router::new()` creates a route that matches `/login`.
                    // If `main.rs` nests it at `/`, then the path is `/login`.
                    // UNLESS `api/mod.rs` does something. I haven't seen `api/mod.rs`.
                    // Let me check `backend/src/api/mod.rs`.
                    // If `login.rs` works with `/admin/login`, then the backend MUST be serving it there.
                    // I will check `backend/src/api/mod.rs` to see if it nests `admin::router`.
                .header("Authorization", &format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .json(&req)
                .unwrap()
                .send()
                .await;

                match resp {
                    Ok(r) => {
                        if r.ok() {
                            set_success.set("Password changed successfully".to_string());
                            set_current_password.set("".to_string());
                            set_new_password.set("".to_string());
                            set_confirm_password.set("".to_string());
                        } else {
                            let text = r.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                            set_error.set(format!("Error: {}", text));
                        }
                    }
                    Err(e) => {
                         set_error.set(format!("Network error: {}", e));
                    }
                }
                set_loading.set(false);
            });
        }
    };

    #[cfg(not(target_arch = "wasm32"))]
    let on_submit = move |_ev: leptos::ev::SubmitEvent| {};

    view! {
        <div class="container py-12">
            <div class="max-w-md mx-auto card">
                <h1 class="text-2xl font-bold mb-6">"Change Password"</h1>

                {move || (!error.get().is_empty()).then(|| view! {
                    <div class="mb-4 p-3 bg-red-50 border border-red-200 text-red-700 rounded-md text-sm">
                        {error.get()}
                    </div>
                })}

                {move || (!success.get().is_empty()).then(|| view! {
                    <div class="mb-4 p-3 bg-green-50 border border-green-200 text-green-700 rounded-md text-sm">
                        {success.get()}
                    </div>
                })}

                <form on:submit=on_submit>
                    <div class="form-group mb-4">
                        <label class="block mb-2 font-bold">"Current Password"</label>
                        <input
                            type="password"
                            class="w-full p-2 border rounded"
                            prop:value=move || current_password.get()
                            on:input=move |ev| set_current_password.set(event_target_value(&ev))
                            disabled=move || loading.get()
                        />
                    </div>
                    <div class="form-group mb-4">
                        <label class="block mb-2 font-bold">"New Password"</label>
                        <input
                            type="password"
                            class="w-full p-2 border rounded"
                            prop:value=move || new_password.get()
                            on:input=move |ev| set_new_password.set(event_target_value(&ev))
                            disabled=move || loading.get()
                        />
                    </div>
                    <div class="form-group mb-6">
                        <label class="block mb-2 font-bold">"Confirm New Password"</label>
                        <input
                            type="password"
                            class="w-full p-2 border rounded"
                            prop:value=move || confirm_password.get()
                            on:input=move |ev| set_confirm_password.set(event_target_value(&ev))
                            disabled=move || loading.get()
                        />
                    </div>
                    <button
                        type="submit"
                        class="btn btn-primary w-full"
                        disabled=move || loading.get()
                    >
                        {move || if loading.get() { "Updating..." } else { "Change Password" }}
                    </button>
                </form>
                <div class="mt-4 text-center">
                    <a href="/admin/dashboard" class="text-blue-600 hover:underline">"Back to Dashboard"</a>
                </div>
            </div>
        </div>
    }
}
