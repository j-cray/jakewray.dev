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

        use axum::Extension;
        use axum::http::HeaderMap;

        let Extension(key): Extension<Key> = extract().await?;
        let headers: HeaderMap = extract().await?;
        let jar = SignedCookieJar::from_headers(&headers, key);
        let pool = use_context::<PgPool>().ok_or(ServerFnError::new("Pool not found"))?;

        if let Some(cookie) = jar.get("auth_token") {
            let user_id = cookie.value();
            match uuid::Uuid::parse_str(user_id) {
                Ok(uuid) => {
                    use sqlx::Row;
                    let user_row = sqlx::query("SELECT username FROM users WHERE id = $1")
                        .bind(uuid)
                        .fetch_optional(&pool)
                        .await
                        .unwrap_or(None);

                    if let Some(row) = user_row {
                        let username: String = row.try_get("username").unwrap_or_default();
                        return Ok(Some(UserResponse { username }));
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
