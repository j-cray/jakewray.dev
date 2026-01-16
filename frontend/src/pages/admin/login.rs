use leptos::logging;
use leptos::prelude::*;
use leptos_router::hooks::*;
use gloo_net::http::Request;

#[component]
pub fn AdminLoginPage() -> impl IntoView {
    let (username, set_username) = signal("".to_string());
<<<<<<< HEAD
    let (password, set_password) = signal("".to_string());
    let (error, set_error) = signal(Option::<String>::None);
=======
    let (_password, set_password) = signal("".to_string());
>>>>>>> origin/main

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error.set(None);

        let user = username.get();
        let pass = password.get();

        leptos::task::spawn_local(async move {
            let resp = Request::post("/api/admin/login")
                .json(&serde_json::json!({
                    "username": user,
                    "password": pass
                }))
                .expect("Failed to create request")
                .send()
                .await;

            match resp {
                Ok(response) if response.ok() => {
                     let navigate = use_navigate();
                     navigate("/admin/dashboard", Default::default());
                }
                _ => {
                    set_error.set(Some("Invalid username or password".to_string()));
                }
            }
        });
    };

    view! {
        <div class="flex items-center justify-center min-h-screen bg-gray-900 text-white">
            <div class="card w-full max-w-md bg-white/5 p-8 rounded-lg shadow-md border border-white/10 glass">
                <h1 class="text-3xl font-bold mb-8 text-center text-white font-heading tracking-wide">"Admin Login"</h1>

                {move || error.get().map(|e| view! {
                    <div class="bg-red-500/10 border border-red-500/30 text-red-200 p-4 rounded-lg mb-6 text-center text-sm">
                        {e}
                    </div>
                })}

                <form on:submit=on_submit class="flex flex-col gap-5">
                    <div class="space-y-1">
                        <label class="text-sm text-gray-400 ml-1">"Username"</label>
                        <input
                            type="text"
                            placeholder="Enter username"
                            class="input"
                            on:input=move |ev| set_username.set(event_target_value(&ev))
                            prop:value=username
                        />
                    </div>

                    <div class="space-y-1">
                        <label class="text-sm text-gray-400 ml-1">"Password"</label>
                        <input
                            type="password"
                            placeholder="Enter password"
                            class="input"
                            on:input=move |ev| set_password.set(event_target_value(&ev))
                            prop:value=password
                        />
                    </div>

                    <button type="submit" class="btn-primary w-full mt-4">
                        "Login"
                    </button>


                     <a href="/" class="text-center text-sm text-gray-500 hover:text-white transition">"Back to Home"</a>
                </form>
            </div>
        </div>
    }
}
