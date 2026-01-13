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
// leptonic imports removed
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::*;
use leptos_router::*;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Html lang="en"/>
        <Head>
            <Title text="Jake Wray"/>
            <Meta name="description" content="Journalist, Programmer, Photographer."/>
            <Meta charset="utf-8"/>
            <Meta name="viewport" content="width=device-width, initial-scale=1"/>
            <Stylesheet id="leptos" href="/pkg/jakewray_ca.css"/>
        </Head>
        <Body>
            <Router>
                <div class="min-h-screen flex flex-col bg-gray-900 text-white">
                    <Navbar/>
                    <main class="flex-grow container mx-auto px-4 py-8">
                        <Routes fallback=|| view! { <NotFound/> }>
                             <Route path=leptos_router::path!("/") view=|| view! { <HomePage/> }/>
                             <Route path=leptos_router::path!("/about") view=|| view! { <AboutPage/> }/>
                             <Route path=leptos_router::path!("/contact") view=|| view! { <ContactPage/> }/>
                             <Route path=leptos_router::path!("/setup") view=|| view! { <crate::pages::setup::SetupPage/> }/>
                             <Route path=leptos_router::path!("/admin") view=|| view! { <AdminLoginPage/> }/>
                             <ParentRoute path=leptos_router::path!("/admin") view=|| view! { <crate::pages::admin::AdminProtectedLayout/> }>
                                <Route path=leptos_router::path!("dashboard") view=|| view! { <AdminDashboard/> }/>
                                <Route path=leptos_router::path!("composer") view=|| view! { <AdminComposer/> }/>
                                <Route path=leptos_router::path!("sync") view=|| view! { <AdminSyncManager/> }/>
                                <Route path=leptos_router::path!("media") view=|| view! { <MediaLibraryPlaceholder/> }/>
                             </ParentRoute>
                             <Route path=leptos_router::path!("/journalism") view=|| view! { <JournalismPage/> }/>
                             <Route path=leptos_router::path!("/personal") view=|| view! { <PersonalPage/> }/>
                             <Route path=leptos_router::path!("/creative-writing") view=|| view! { <CreativeWritingPage/> }/>
                             <Route path=leptos_router::path!("/music") view=|| view! { <MusicPage/> }/>
                             <Route path=leptos_router::path!("/visual-art") view=|| view! { <VisualArtPage/> }/>
                             <Route path=leptos_router::path!("/programming") view=|| view! { <ProgrammingPage/> }/>
                        </Routes>
                    </main>
                    <Footer/>
                </div>
            </Router>
            <Scripts/>
        </Body>
    }
}

#[component]
fn SectionLayout() -> impl IntoView {
    view! { <Outlet/> }
}

#[component]
fn MediaLibraryPlaceholder() -> impl IntoView {
    view! { "Media Library Placeholder" }
}

#[component]
fn HomePage() -> impl IntoView {
    view! {
        <div class="p-8 max-w-[1400px] mx-auto flex flex-col gap-16">
            <header class="text-center py-16">
                <h1 class="mb-2 text-6xl font-extrabold tracking-tighter">
                    <span class="text-gradient">"JAKE WRAY"</span>
                </h1>
                <p class="text-2xl text-gray-400 max-w-[600px] mx-auto">
                    "Journalist. Developer. Photographer. Creating extensive archives of the present."
                </p>
                <div class="mt-8 flex gap-4 justify-center">
                    <button on:click=move |_| {} class="btn-filled text-lg px-8 py-3 rounded-full">
                        "Read Journal"
                    </button>
                    <button on:click=move |_| {} class="btn-outlined text-lg px-8 py-3 rounded-full border border-white/20 hover:bg-white/10 transition">
                        "View Portfolio"
                    </button>
                </div>
            </header>

            <div class="grid grid-cols-1 sm:grid-cols-3 gap-8">
                <div class="h-full">
                    <div class="glass h-full transition-transform hover:scale-105 p-6 rounded-xl border border-white/10">
                        <h3 class="font-bold text-brand text-xl mb-2">"Latest Articles"</h3>
                        <p class="text-gray-300">"Deep dives into technology, culture, and the intersection of both."</p>
                    </div>
                </div>
                <div class="h-full">
                    <div class="glass h-full transition-transform hover:scale-105 p-6 rounded-xl border border-white/10">
                        <h3 class="font-bold text-brand text-xl mb-2">"Recent Projects"</h3>
                        <p class="text-gray-300">"Software engineering experiments, open source contributions, and more."</p>
                    </div>
                </div>
                <div class="h-full">
                    <div class="glass h-full transition-transform hover:scale-105 p-6 rounded-xl border border-white/10">
                        <h3 class="font-bold text-brand text-xl mb-2">"Visuals"</h3>
                        <p class="text-gray-300">"A collection of photography and digital art capturing moments in time."</p>
                    </div>
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
