#[cfg(feature = "hydrate")]
use gloo_net::http::Request;
use leptos::prelude::*;
#[cfg(feature = "hydrate")]
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

    let location = use_location();

    Effect::new(move || {
        if let Some(err) = location.query.get().get("error") {
            if err == "invalid" {
                set_error.set("Invalid username or password.".to_string());
            }
        }
    });

    #[cfg(feature = "hydrate")]
    let on_submit = {
        let navigate = use_navigate();
        move |ev: leptos::ev::SubmitEvent| {
            ev.prevent_default();
            web_sys::console::log_1(&"[Login] Form submitted".into());
            set_loading.set(true);
            set_error.set("".to_string());

            let username_val = username.get();
            let password_val = password.get();
            let navigate = navigate.clone();

            web_sys::console::log_1(&format!("[Login] Attempting login for user: {}", username_val).into());

            spawn_local(async move {
                let req = LoginRequest {
                    username: username_val.clone(),
                    password: password_val.clone(),
                };

                web_sys::console::log_1(&"[Login] Sending POST /admin/login".into());

                let result = async {
                    let resp = Request::post("/admin/login")
                        .header("Content-Type", "application/json")
                        .json(&req)
                        .map_err(|e| {
                            web_sys::console::log_1(&format!("[Login] Serialize error: {:?}", e).into());
                            "Failed to serialize request".to_string()
                        })?
                        .send()
                        .await
                        .map_err(|e| {
                            web_sys::console::log_1(&format!("[Login] Network error: {:?}", e).into());
                            "Failed to connect to server".to_string()
                        })?;

                    web_sys::console::log_1(&format!("[Login] Response status: {}", resp.status()).into());

                    if !resp.ok() {
                        return Err("Invalid username or password".to_string());
                    }

                    let data: LoginResponse = resp
                        .json()
                        .await
                        .map_err(|e| {
                            web_sys::console::log_1(&format!("[Login] Parse error: {:?}", e).into());
                            "Failed to parse response".to_string()
                        })?;

                    web_sys::console::log_1(&"[Login] Token received, storing in localStorage".into());

                    // Store token in localStorage
                    let window = web_sys::window().unwrap();
                    let local_storage = window.local_storage().unwrap().unwrap();
                    let _ = local_storage.set_item("admin_token", &data.token);

                    Ok(())
                }
                .await;

                match result {
                    Ok(()) => {
                        web_sys::console::log_1(&"[Login] Success, navigating to dashboard".into());
                        navigate("/admin/dashboard", Default::default())
                    },
                    Err(msg) => {
                        web_sys::console::log_1(&format!("[Login] Error: {}", msg).into());
                        set_error.set(msg);
                    }
                }

                set_loading.set(false);
            });
        }
    };

    #[cfg(not(feature = "hydrate"))]
    let on_submit = move |_ev: leptos::ev::SubmitEvent| {};

    view! {
        <div class="center-page">
            <div class="card form-card">
                <h1 class="mb-2 text-primary text-center">"Admin Access"</h1>
                <p class="text-muted text-center mb-6">"Secure dashboard login"</p>
                
                {move || {
                    (!error.get().is_empty()).then(|| view! {
                        <div class="mb-4 p-3 bg-red-50 border border-red-200 text-red-700 rounded-md text-sm">
                            {error.get()}
                        </div>
                    })
                }}

                <form autocomplete="on" method="post" action="/admin/login" on:submit=on_submit>
                    <div class="form-group">
                        <label for="username">"Username"</label>
                        <input
                            id="username"
                            name="username"
                            type="text"
                            placeholder="Enter username"
                            autocomplete="username"
                            on:input=move |ev| set_username.set(event_target_value(&ev))
                            disabled=move || loading.get()
                        />
                    </div>
                    
                    <div class="form-group">
                        <label for="password">"Password"</label>
                        <input
                            id="password"
                            name="password"
                            type="password"
                            placeholder="Enter password"
                            autocomplete="current-password"
                            on:input=move |ev| set_password.set(event_target_value(&ev))
                            disabled=move || loading.get()
                        />
                    </div>
                    
                    <button type="submit" class="btn btn-primary" disabled=move || loading.get()>
                        {move || if loading.get() { "Logging in..." } else { "Login" }}
                    </button>
                </form>
            </div>
        </div>
    }
}
