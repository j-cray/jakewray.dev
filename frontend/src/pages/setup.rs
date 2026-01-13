use leptos::prelude::*;
use leptos_router::hooks::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct SetupStatus {
    required: bool,
}

#[server(GetSetupStatus, "/api/setup/status")]
#[allow(unused_variables)]
pub async fn get_setup_status() -> Result<SetupStatus, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use sqlx::PgPool;
        let pool = use_context::<PgPool>().ok_or(ServerFnError::new("Pool not found"))?;

        use sqlx::Row;
        let count: i64 = sqlx::query("SELECT COUNT(*) as count FROM users")
            .fetch_one(&pool)
            .await
            .map(|r| r.try_get("count").unwrap_or(0))
            .unwrap_or(0);

        Ok(SetupStatus { required: count == 0 })
    }
    #[cfg(not(feature = "ssr"))]
    {
        unreachable!()
    }
}

#[server(PerformSetup, "/api/setup")]
#[allow(unused_variables)]
pub async fn perform_setup(username: String, password: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use sqlx::PgPool;
        use bcrypt::{hash, DEFAULT_COST};

        let pool = use_context::<PgPool>().ok_or(ServerFnError::new("Pool not found"))?;

        use sqlx::Row;
        // 1. Verify no users exist
        let count: i64 = sqlx::query("SELECT COUNT(*) as count FROM users")
            .fetch_one(&pool)
            .await
            .map(|r| r.try_get("count").unwrap_or(0))
            .unwrap_or(0);

        if count > 0 {
            return Err(ServerFnError::new("Setup already completed"));
        }

        // 2. Create user
        // 2. Create user
        let hashed_password = hash(&password, DEFAULT_COST)?;

        sqlx::query(
            "INSERT INTO users (username, password_hash) VALUES ($1, $2)",
        )
        .bind(username)
        .bind(hashed_password)
        .execute(&pool)
        .await?;

        Ok(())
    }
    #[cfg(not(feature = "ssr"))]
    {
        unreachable!()
    }
}

#[component]
pub fn SetupPage() -> impl IntoView {
    let (username, set_username) = signal("".to_string());
    let (password, set_password) = signal("".to_string());
    let (confirm_password, set_confirm_password) = signal("".to_string());
    let (error, set_error) = signal(Option::<String>::None);

    // Check if setup is actually required
    let setup_status = Resource::new(|| (), |_| async move {
        get_setup_status().await.ok()
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error.set(None);

        let user = username.get();
        let pass = password.get();
        let confirm = confirm_password.get();

        if pass != confirm {
             set_error.set(Some("Passwords do not match".to_string()));
             return;
        }

        if user.is_empty() || pass.is_empty() {
            set_error.set(Some("All fields are required".to_string()));
            return;
        }

        leptos::task::spawn_local(async move {
            match perform_setup(user, pass).await {
                Ok(_) => {
                     let navigate = use_navigate();
                     // Redirect to login after successful creation
                     navigate("/admin", Default::default());
                }
                Err(e) => {
                    set_error.set(Some(format!("Failed to create admin user: {}", e)));
                }
            }
        });
    };

    view! {
        <Suspense fallback=move || view! { <div class="text-white text-center pt-20">"Checking setup status..."</div> }>
            {move || match setup_status.get() {
                 Some(Some(status)) if status.required => view! {
                    <div class="flex items-center justify-center min-h-screen bg-gray-900 text-white">
                        <div class="card w-full max-w-md bg-white/5 p-8 rounded-lg shadow-md border border-white/10 glass">
                            <h1 class="text-3xl font-bold mb-2 text-center text-brand">"Welcome"</h1>
                            <p class="text-gray-400 text-center mb-8">"Create your owner account to get started."</p>

                            {move || error.get().map(|e| view! {
                                <div class="bg-red-500/20 text-red-200 p-3 rounded mb-4 text-center border border-red-500/50">
                                    {e}
                                </div>
                            })}

                            <form on:submit=on_submit class="flex flex-col gap-4">
                                <input
                                    type="text"
                                    placeholder="Username"
                                    class="p-2 rounded-md bg-black/50 border border-white/10 text-white focus:border-brand focus:outline-none"
                                    on:input=move |ev| set_username.set(event_target_value(&ev))
                                    prop:value=username
                                />
                                <input
                                    type="password"
                                    placeholder="Password"
                                    class="p-2 rounded-md bg-black/50 border border-white/10 text-white focus:border-brand focus:outline-none"
                                    on:input=move |ev| set_password.set(event_target_value(&ev))
                                     prop:value=password
                                />
                                <input
                                    type="password"
                                    placeholder="Confirm Password"
                                    class="p-2 rounded-md bg-black/50 border border-white/10 text-white focus:border-brand focus:outline-none"
                                    on:input=move |ev| set_confirm_password.set(event_target_value(&ev))
                                     prop:value=confirm_password
                                />
                                <button type="submit" class="bg-brand text-black p-2 rounded-md font-bold hover:bg-brand-dim transition mt-2">
                                    "Create Account"
                                </button>
                            </form>
                        </div>
                    </div>
                 }.into_any(),
                 Some(Some(_)) => view! {
                     <div class="text-center pt-20 text-white">
                        "Setup already completed. " <a href="/admin" class="text-brand">"Go to Login"</a>
                     </div>
                 }.into_any(),
                 _ => view! { <div class="text-center pt-20 text-red-400">"Error checking status"</div> }.into_any()
            }}
        </Suspense>
    }
}
