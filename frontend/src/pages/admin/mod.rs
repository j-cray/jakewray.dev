pub mod login;
pub mod dashboard;
pub mod composer;
pub mod sync_manager;

use leptos::prelude::*;
use leptos_router::components::{Outlet, Redirect};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub username: String,
}

#[server(GetCurrentUser, "/api/admin/me")]
pub async fn get_current_user() -> Result<Option<UserResponse>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum_extra::extract::cookie::{Cookie, SignedCookieJar};
        use leptos_axum::extract;
        use sqlx::PgPool;
        use axum_extra::extract::cookie::Key;

        let jar: SignedCookieJar = extract().await?;
        let pool = use_context::<PgPool>().ok_or(ServerFnError::ServerError("Pool not found".to_string()))?;

        if let Some(cookie) = jar.get("auth_token") {
            let user_id = cookie.value();
            match uuid::Uuid::parse_str(user_id) {
                Ok(uuid) => {
                    let user = sqlx::query!("SELECT username FROM users WHERE id = $1", uuid)
                        .fetch_optional(&pool)
                        .await
                        .unwrap_or(None);

                    if let Some(u) = user {
                        return Ok(Some(UserResponse { username: u.username }));
                    }
                }
                Err(_) => return Ok(None),
            };
        }

        Ok(None)
    }
    #[cfg(not(feature = "ssr"))]
    {
        unreachable!()
    }
}

#[component]
pub fn AdminProtectedLayout() -> impl IntoView {
    // Resource to check if user is authenticated
    let auth_status = Resource::new(
        || (),
        |_| async move {
            get_current_user().await.ok().flatten().is_some()
        },
    );

    view! {
        <Suspense fallback=move || view! {
            <div class="flex items-center justify-center min-h-screen bg-gray-900 text-white">
                "Verifying authentication..."
            </div>
        }>
            {move || match auth_status.get() {
                Some(true) => view! {
                    <div class="admin-layout">
                        <Outlet/>
                    </div>
                }.into_any(),
                Some(false) => view! { <Redirect path="/admin"/> }.into_any(), // Redirect to login
                None => view! { }.into_any(),
            }}
        </Suspense>
    }
}
