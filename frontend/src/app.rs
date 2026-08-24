use crate::components::admin_bar::AdminBar;
use crate::components::footer::Footer;
use crate::components::navbar::Navbar;
use crate::context::provide_admin_context;
use crate::pages::about::AboutPage;
use crate::pages::admin::composer::AdminComposer;
use crate::pages::admin::dashboard::AdminDashboard;
use crate::pages::admin::login::AdminLoginPage;
use crate::pages::admin::media::AdminMedia;
use crate::pages::admin::password_change::AdminPasswordChange;
use crate::pages::blog::PersonalBlogPage;
use crate::pages::home::{AdminRedirect, HomePage, NotFound};
use crate::pages::journalism::{JournalismArticlePage, JournalismPage};
use crate::pages::programming::ProgrammingPage;

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::*;
use leptos_router::hooks::use_location;
use leptos_router::*;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <MainLayout/>
        </Router>
    }
}

#[component]
pub fn Shell() -> impl IntoView {
    provide_meta_context();
    let options =
        use_context::<leptos::config::LeptosOptions>().expect("LeptosOptions not found in Shell");

    #[cfg(feature = "ssr")]
    let ga_config = {
        let is_prod = std::env::var("ENVIRONMENT").as_deref() == Ok("production");
        let ga_id = std::env::var("GA_MEASUREMENT_ID").unwrap_or_default();
        if is_prod && !ga_id.is_empty() {
            let script_src = format!("https://www.googletagmanager.com/gtag/js?id={}", ga_id);
            let inline_script = format!(
                "window.dataLayer = window.dataLayer || [];\n\
                 function gtag(){{dataLayer.push(arguments);}}\n\
                 gtag('js', new Date());\n\
                 gtag('config', '{}', {{ 'send_page_view': false }});",
                ga_id
            );
            Some((script_src, inline_script))
        } else {
            None
        }
    };

    view! {
        <html lang="en">
            <head>
                <Meta charset="utf-8"/>
                <Meta name="viewport" content="width=device-width, initial-scale=1"/>
                <Meta name="description" content="Journalist, Programmer, Photographer."/>
                <Title text="Jake Wray"/>
                <Stylesheet id="leptos" href="/pkg/jakewray_ca.css"/>
                <MetaTags/>
                {
                    #[cfg(feature = "ssr")]
                    {
                        if let Some((src, inline_js)) = ga_config {
                            view! {
                                <script async src=src></script>
                                <script>{inline_js}</script>
                            }.into_any()
                        } else {
                            view! { <span class="hidden" /> }.into_any()
                        }
                    }
                    #[cfg(not(feature = "ssr"))]
                    {
                        view! { <span class="hidden" /> }.into_any()
                    }
                }
            </head>
            <body><App/><HydrationScripts options=options/></body>
        </html>
    }
}

#[component]
fn MainLayout() -> impl IntoView {
    let admin_ctx = provide_admin_context();
    let location = use_location();

    Effect::new(move || {
        admin_ctx.init_from_storage();
    });

    Effect::new(move || {
        let _path = location.pathname.get();
        admin_ctx.clear_action();
    });

    Effect::new(move || {
        let path = location.pathname.get();
        let is_admin = admin_ctx.is_admin.get();

        #[cfg(target_arch = "wasm32")]
        let title = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| js_sys::Reflect::get(&d, &wasm_bindgen::JsValue::from_str("title")).ok())
            .and_then(|t| t.as_string())
            .unwrap_or_else(|| "Jake Wray".to_string());
        #[cfg(not(target_arch = "wasm32"))]
        let title = "Jake Wray".to_string();

        crate::utils::analytics::track_page_view(&path, &title, is_admin);
    });

    let theme_class = move || {
        let path = location.pathname.get();
        if path.starts_with("/code") {
            "theme-code"
        } else if path.starts_with("/blog") {
            "theme-blog"
        } else if path.starts_with("/journalism") {
            "theme-journalism"
        } else if path.starts_with("/about") {
            "theme-about"
        } else {
            ""
        }
    };

    view! {
        <div class=move || {
            let admin_extra = if admin_ctx.is_admin.get() { " has-admin-bar" } else { "" };
            format!("min-h-screen flex flex-col bg-gray-50/50 {}{}", theme_class(), admin_extra)
        }>
            <Navbar/>
            <main class="flex-grow p-4">
                <Routes fallback=|| view! { <NotFound/> }>
                    // Public Routes
                    <Route path=path!("/") view=HomePage/>
                    <Route path=path!("/code") view=ProgrammingPage/>
                    <Route path=path!("/journalism") view=JournalismPage/>
                    <Route path=path!("/journalism/:slug") view=JournalismArticlePage/>
                    <Route path=path!("/blog") view=PersonalBlogPage/>
                    <Route path=path!("/about") view=AboutPage/>

                    // Admin Routes
                    <Route path=path!("/admin") view=AdminRedirect/>
                    <Route path=path!("/admin/dashboard") view=AdminDashboard/>
                    <Route path=path!("/admin/login") view=AdminLoginPage/>
                    <Route path=path!("/admin/compose") view=AdminComposer/>
                    <Route path=path!("/admin/password-change") view=AdminPasswordChange/>
                    <Route path=path!("/admin/media") view=AdminMedia/>
                </Routes>
            </main>
            <Footer/>
            <AdminBar/>
        </div>
    }
}
