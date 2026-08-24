use crate::context::{get_default_contextual_action, use_admin_context, AdminActionIcon};
use leptos::prelude::*;
use leptos_router::hooks::{use_location, use_navigate};

#[component]
pub fn AdminBar() -> impl IntoView {
    let admin_ctx = use_admin_context();
    let location = use_location();
    let navigate = use_navigate();

    view! {
        {move || {
            if !admin_ctx.is_admin.get() {
                return view! { <span class="hidden" /> }.into_any();
            }

            let path = location.pathname.get();
            let custom_action = admin_ctx.contextual_action.get();
            let action = custom_action.or_else(|| get_default_contextual_action(&path));
            let is_on_admin_dashboard = path == "/admin/dashboard";

            let on_logout = {
                let location = location.clone();
                let navigate = navigate.clone();
                move |_| {
                    let current_path = location.pathname.get();
                    admin_ctx.logout();
                    if current_path.starts_with("/admin") && current_path != "/admin/login" {
                        navigate("/admin/login", Default::default());
                    }
                }
            };

            view! {
                <div class="admin-bar-wrapper">
                    <nav class="admin-bar" aria-label="Admin quick actions">
                        // Admin Mode Badge
                        <div class="admin-bar-badge" title="Admin mode active">
                            <span class="admin-bar-dot" aria-hidden="true"></span>
                            <span>"Admin"</span>
                        </div>

                        // Contextual Action Button (if any)
                        {action.map(|act| {
                            let is_active = act.is_active;
                            let btn_class = if is_active {
                                "admin-bar-btn btn-contextual is-active"
                            } else {
                                "admin-bar-btn btn-contextual"
                            };
                            let label = act.label.clone();
                            let icon = act.icon;

                            let icon_view = move || match icon {
                                AdminActionIcon::Edit => view! {
                                    <svg xmlns="http://www.w3.org/2000/svg" class="admin-bar-icon" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                                    </svg>
                                }.into_any(),
                                AdminActionIcon::Compose => view! {
                                    <svg xmlns="http://www.w3.org/2000/svg" class="admin-bar-icon" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
                                    </svg>
                                }.into_any(),
                                AdminActionIcon::Dashboard => view! {
                                    <svg xmlns="http://www.w3.org/2000/svg" class="admin-bar-icon" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zM14 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zM14 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z" />
                                    </svg>
                                }.into_any(),
                                AdminActionIcon::Media => view! {
                                    <svg xmlns="http://www.w3.org/2000/svg" class="admin-bar-icon" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
                                    </svg>
                                }.into_any(),
                                AdminActionIcon::Close => view! {
                                    <svg xmlns="http://www.w3.org/2000/svg" class="admin-bar-icon" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                                    </svg>
                                }.into_any(),
                            };

                            view! {
                                <div class="admin-bar-divider" aria-hidden="true"></div>
                                {if let Some(cb) = act.on_click {
                                    view! {
                                        <button
                                            type="button"
                                            class=btn_class
                                            on:click=move |_| cb.run(())
                                        >
                                            {icon_view()}
                                            <span>{label}</span>
                                        </button>
                                    }.into_any()
                                } else if let Some(href) = act.href {
                                    view! {
                                        <a
                                            href=href
                                            class=btn_class
                                        >
                                            {icon_view()}
                                            <span>{label}</span>
                                        </a>
                                    }.into_any()
                                } else {
                                    view! {
                                        <button
                                            type="button"
                                            class=btn_class
                                            disabled=true
                                        >
                                            {icon_view()}
                                            <span>{label}</span>
                                        </button>
                                    }.into_any()
                                }}
                            }
                        })}

                        <div class="admin-bar-divider" aria-hidden="true"></div>

                        // Universal Action 1: Admin Panel Link
                        {if is_on_admin_dashboard {
                            view! {
                                <a
                                    href="/"
                                    class="admin-bar-btn btn-panel"
                                    title="View public website"
                                >
                                    <svg xmlns="http://www.w3.org/2000/svg" class="admin-bar-icon" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
                                    </svg>
                                    <span>"View Site"</span>
                                </a>
                            }.into_any()
                        } else {
                            view! {
                                <a
                                    href="/admin/dashboard"
                                    class="admin-bar-btn btn-panel"
                                    title="Open Admin Dashboard"
                                >
                                    <svg xmlns="http://www.w3.org/2000/svg" class="admin-bar-icon" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zM14 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zM14 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z" />
                                    </svg>
                                    <span>"Admin Panel"</span>
                                </a>
                            }.into_any()
                        }}

                        // Universal Action 2: Logout Button
                        <button
                            type="button"
                            class="admin-bar-btn btn-logout"
                            on:click=on_logout
                            title="Log out of admin mode"
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" class="admin-bar-icon" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" />
                            </svg>
                            <span>"Logout"</span>
                        </button>
                    </nav>
                </div>
            }.into_any()
        }}
    }
}
