use crate::components::footer::Footer;
use crate::components::navbar::Navbar;
use crate::pages::admin::composer::AdminComposer;
use crate::pages::admin::dashboard::AdminDashboard;
use crate::pages::admin::login::AdminLoginPage;
use crate::pages::admin::sync_manager::AdminSyncManager;
use crate::pages::about::AboutPage;
use crate::pages::sections::{
    JournalismArticlePage, JournalismPage, PersonalBlogPage, ProgrammingPage,
};
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::*;
use leptos_router::*;
use leptos_router::hooks::use_location;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Router>
            <MainLayout/>
        </Router>
    }
}

#[component]
pub fn Shell() -> impl IntoView {
    let options = use_context::<leptos::config::LeptosOptions>().expect("LeptosOptions not found in Shell");
    view! {
        <html lang="en">
            <head>
                <Meta charset="utf-8"/>
                <Meta name="viewport" content="width=device-width, initial-scale=1"/>
                <Meta name="description" content="Journalist, Programmer, Photographer."/>
                <Title text="Jake Wray"/>
                <Stylesheet id="leptos" href="/pkg/jakewray_ca.css"/>
                <MetaTags/>
            </head>
            <body><App/><HydrationScripts options=options/></body>
        </html>
    }
}


#[component]
fn MainLayout() -> impl IntoView {
    let location = use_location();
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
        <div class=move || format!("min-h-screen flex flex-col bg-gray-50/50 {}", theme_class())>
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
                    <Route path=path!("/admin/sync") view=AdminSyncManager/>
                    <Route path=path!("/admin/media") view=MediaLibraryPlaceholder/>
                </Routes>
            </main>
            <Footer/>
        </div>
    }
}

#[component]
fn SectionLayout() -> impl IntoView {
    view! { <Outlet/> }
}

#[component]
fn AdminRedirect() -> impl IntoView {
    let navigate = leptos_router::hooks::use_navigate();
    leptos::prelude::Effect::new(move || {
        navigate("/admin/login", Default::default());
    });
    view! {}
}

#[component]
fn MediaLibraryPlaceholder() -> impl IntoView {
    view! { "Media Library Placeholder" }
}

#[component]
fn HomePage() -> impl IntoView {
    view! {
        <div class="container home-hero">
            <header class="hero">
                <h1 class="hero-title">"Jake Wray"</h1>
                <p class="hero-subtitle">
                    "A work in progress (me and the website)"
                </p>
            </header>

            <div class="card-grid">
                <div class="card">
                    <h3 class="text-xl font-bold">"Latest Articles"</h3>
                    <p class="text-muted">"Coming soon..."</p>
                </div>
                <div class="card">
                    <h3 class="text-xl font-bold">"Recent Projects"</h3>
                    <p class="text-muted">"Coming soon..."</p>
                </div>
                <div class="card">
                    <h3 class="text-xl font-bold">"Visuals"</h3>
                    <p class="text-muted">"Coming soon..."</p>
                </div>
            </div>
        </div>
    }
}

#[component]
fn NotFound() -> impl IntoView {
    view! {
        <div class="container py-24 text-center">
            <h1 class="text-4xl mb-4">"404"</h1>
            <p>"Page not found."</p>
        </div>
    }
}

#[component]
fn DummyPage() -> impl IntoView {
    view! { "Dummy" }
}
