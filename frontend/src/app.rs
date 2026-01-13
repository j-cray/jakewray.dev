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
use leptonic::components::button::{Button, ButtonSize, ButtonVariant};
use leptonic::components::card::Card;
use leptonic::components::grid::{Col, Grid, Row};
use leptonic::components::root::Root;
use leptonic::components::stack::{Stack, StackOrientation};
use leptonic::components::theme::LeptonicTheme;
use leptonic::components::typography::{H1, H3, P};
use leptonic::prelude::*;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::*;
use leptos_router::hooks::*;
use leptos_router::*;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <html lang="en">
        <head>
            <Title text="Jake Wray"/>
            <Meta name="description" content="Journalist, Programmer, Photographer."/>
            <Meta charset="utf-8"/>
            <Meta name="viewport" content="width=device-width, initial-scale=1"/>
            <Stylesheet id="leptos" href="/pkg/jakewray_ca.css"/>
        </head>
        <body>
            <Root default_theme=LeptonicTheme::Dark>
                <Router>
                    <div class="min-h-screen flex flex-col bg-gray-50/50">
                        <Navbar/>
                        <main class="flex-grow container mx-auto px-4 py-8">
                            <Routes fallback=|| view! { <NotFound/> }>
                                <Route path="/" view=HomePage/>
                                <Route path="/about" view=AboutPage/>
                                <Route path="/contact" view=ContactPage/>
                                <Route path="/admin" view=AdminLoginPage/>
                                <Route path="/admin/dashboard" view=AdminDashboard/>
                                <Route path="/admin/composer" view=AdminComposer/>
                                <Route path="/admin/sync" view=AdminSyncManager/>
                                <Route path="/admin/media" view=MediaLibraryPlaceholder/>
                                <Route path="/journalism" view=JournalismPage/>
                                <Route path="/personal" view=PersonalPage/>
                                <Route path="/creative-writing" view=CreativeWritingPage/>
                                <Route path="/music" view=MusicPage/>
                                <Route path="/visual-art" view=VisualArtPage/>
                                <Route path="/programming" view=ProgrammingPage/>
                            </Routes>
                        </main>
                        <Footer/>
                    </div>
                </Router>
            </Root>
        </body>
        </html>
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
        <Stack orientation=StackOrientation::Vertical spacing=Size::Em(4.0) style="padding: 2em; max-width: 1400px; margin: 0 auto;">
            <header style="text-align: center; padding: 4em 0;">
                <H1 style="margin-bottom: 0.5em; font-size: 3.5em; font-weight: 800; letter-spacing: -0.05em;">
                    <span class="text-gradient">"JAKE WRAY"</span>
                </H1>
                <P style="font-size: 1.5em; color: var(--text-muted); max-width: 600px; margin: 0 auto;">
                    "Journalist. Developer. Photographer. Creating extensive archives of the present."
                </P>
                <div style="margin-top: 2em; display: flex; gap: 1em; justify-content: center;">
                    <Button variant=ButtonVariant::Filled size=ButtonSize::Big>
                        "Read Journal"
                    </Button>
                    <Button variant=ButtonVariant::Outlined size=ButtonSize::Big>
                        "View Portfolio"
                    </Button>
                </div>
            </header>

            <Grid spacing=Size::Em(2.0)>
                <Row>
                    <Col xs=12 sm=4>
                        <Card class="glass" style="height: 100%; transition: transform 0.2s;">
                            <H3 style="font-weight: bold; color: var(--brand-color);">"Latest Articles"</H3>
                            <P>"Deep dives into technology, culture, and the intersection of both."</P>
                        </Card>
                    </Col>
                    <Col xs=12 sm=4>
                        <Card class="glass" style="height: 100%; transition: transform 0.2s;">
                            <H3 style="font-weight: bold; color: var(--brand-color);">"Recent Projects"</H3>
                            <P>"Software engineering experiments, open source contributions, and more."</P>
                        </Card>
                    </Col>
                    <Col xs=12 sm=4>
                        <Card class="glass" style="height: 100%; transition: transform 0.2s;">
                            <H3 style="font-weight: bold; color: var(--brand-color);">"Visuals"</H3>
                            <P>"A collection of photography and digital art capturing moments in time."</P>
                        </Card>
                    </Col>
                </Row>
            </Grid>
        </Stack>
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
