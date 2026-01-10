use crate::components::footer::Footer;
use crate::components::navbar::Navbar;
use crate::pages::about::AboutPage;
use crate::pages::admin::composer::AdminComposer;
use crate::pages::admin::dashboard::AdminDashboard;
use crate::pages::admin::login::AdminLoginPage;
use crate::pages::admin::sync_manager::AdminSyncManager;
use crate::pages::contact::ContactPage;
use crate::pages::sections::*;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::*;
use leptos_router::hooks::*;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/jakewray_ca.css"/>
        <Title text="Jake Wray"/>
        <Meta name="description" content="Journalist, Programmer, Photographer."/>

        <Router>
            <div class="min-h-screen flex flex-col">
                <Navbar/>
                <main class="flex-grow">
                    <Routes>
                        // Public Routes
                        <Route path="/" view=HomePage/>
                        <Route path="/about" view=AboutPage/>
                        <Route path="/contact" view=ContactPage/>
                        <Route path="/journalism" view=JournalismPage/>
                        <Route path="/personal" view=PersonalPage/>
                        <Route path="/creative-writing" view=CreativeWritingPage/>
                        <Route path="/music" view=MusicPage/>
                        <Route path="/visual-art" view=VisualArtPage/>
                        <Route path="/programming" view=ProgrammingPage/>

                        // Admin Routes
                        <Route path="/admin" view=AdminDashboard/> // Should redirect to login if not auth
                        <Route path="/admin/login" view=AdminLoginPage/>
                        <Route path="/admin/compose" view=AdminComposer/>
                        <Route path="/admin/sync" view=AdminSyncManager/>
                        <Route path="/admin/media" view=move || view! { "Media Library Placeholder" }/>

                        <Route path="/*any" view=NotFound/>
                    </Routes>
                </main>
                <Footer/>
            </div>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    view! {
        <div class="container py-12">
            <header class="text-center mb-16">
                <h1 class="text-6xl mb-6">"Jake Wray"</h1>
                <p class="text-xl text-muted max-w-2xl mx-auto">
                    "Journalist. Developer. Photographer. Creating extensive archives of the present."
                </p>
            </header>

            <div class="grid grid-cols-1 md:grid-cols-3 gap-8">
                 <div class="card">
                    <h3 class="text-xl mb-4">"Latest Articles"</h3>
                    <p class="text-muted">"Coming soon..."</p>
                 </div>
                 <div class="card">
                    <h3 class="text-xl mb-4">"Recent Projects"</h3>
                    <p class="text-muted">"Coming soon..."</p>
                 </div>
                 <div class="card">
                    <h3 class="text-xl mb-4">"Visuals"</h3>
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
