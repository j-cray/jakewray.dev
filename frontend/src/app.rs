use crate::components::footer::Footer;
use crate::components::navbar::Navbar;
use crate::pages::about::AboutPage;
use crate::pages::admin::composer::AdminComposer;
use crate::pages::admin::dashboard::AdminDashboard;
use crate::pages::admin::login::AdminLoginPage;
use crate::pages::admin::sync_manager::AdminSyncManager;
use crate::pages::contact::ContactPage;
use crate::pages::sections::{
    CreativeWritingPage, JournalismPage, MusicPage, PersonalPage, ProgrammingPage, VisualArtPage,
};
use leptos::config::LeptosOptions;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::*;
use leptos_router::*;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    let options = expect_context::<LeptosOptions>();

    view! {
        <Html attr:lang="en"/>
        <Meta charset="utf-8"/>
        <Title text="Jake Wray"/>
        <Meta name="description" content="Journalist, Programmer, Photographer."/>
        <Meta name="viewport" content="width=device-width, initial-scale=1"/>
        <Stylesheet id="leptos" href="/pkg/jakewray_ca.css"/>

        <Router>
            <div class="min-h-screen flex flex-col bg-gray-50/50">
                <Navbar/>
                <main class="flex-grow p-4">
                    <Routes fallback=|| view! { <NotFound/> }>
                        // Public Routes
                        <Route path=path!("/") view=HomePage/>
                        <Route path=path!("/about") view=AboutPage/>
                        <Route path=path!("/contact") view=ContactPage/>

                        // Portfolio
                        <Route path=path!("/journalism") view=JournalismPage/>
                        <Route path=path!("/personal") view=PersonalPage/>
                        <Route path=path!("/creative-writing") view=CreativeWritingPage/>
                        <Route path=path!("/music") view=MusicPage/>
                        <Route path=path!("/visual-art") view=VisualArtPage/>
                        <Route path=path!("/programming") view=ProgrammingPage/>

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
        </Router>

        <HydrationScripts options=options/>
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
        <div class="container mx-auto py-12">
            <header class="text-center mb-16">
                <h1 class="text-6xl mb-6 font-heading">"Jake Wray"</h1>
                <p class="text-xl text-gray-500 max-w-2xl mx-auto">
                    "Journalist. Developer. Photographer. Creating extensive archives of the present."
                </p>
            </header>

            <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                <div class="p-6 bg-white rounded-xl shadow-sm border border-gray-100">
                    <h3 class="text-xl mb-2 font-bold">"Latest Articles"</h3>
                    <p class="text-gray-500">"Coming soon..."</p>
                </div>
                <div class="p-6 bg-white rounded-xl shadow-sm border border-gray-100">
                    <h3 class="text-xl mb-2 font-bold">"Recent Projects"</h3>
                    <p class="text-gray-500">"Coming soon..."</p>
                </div>
                <div class="p-6 bg-white rounded-xl shadow-sm border border-gray-100">
                    <h3 class="text-xl mb-2 font-bold">"Visuals"</h3>
                    <p class="text-gray-500">"Coming soon..."</p>
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
