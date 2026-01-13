use leptonic::prelude::*;
use leptos::prelude::*;
use leptos::*;

#[component]
pub fn Navbar() -> impl IntoView {
    let navigate = leptos_router::hooks::use_navigate();

    let nav_btn = move |text: &'static str, path: &'static str| {
        let navigate = navigate.clone();
        view! {
            <Button on_click=move |_| navigate(path, Default::default()) variant=ButtonVariant::Flat>
                {text}
            </Button>
        }
    };

    view! {
        <AppBar>
            <Stack orientation=StackOrientation::Horizontal spacing=Size::Em(1.0) style="width: 100%; justify-content: space-between; align-items: center; padding: 0 1em;">
                <H3 style="margin: 0">
                    <Link href="/">"Jake Wray"</Link>
                </H3>

                <Stack orientation=StackOrientation::Horizontal spacing=Size::Em(0.5)>
                    {nav_btn("Journalism", "/journalism")}
                    {nav_btn("Personal", "/personal")}
                    {nav_btn("Writing", "/creative-writing")}
                    {nav_btn("Music", "/music")}
                    {nav_btn("Art", "/visual-art")}
                    {nav_btn("Code", "/programming")}
                    <Separator orientation=SeparatorOrientation::Vertical style="height: 1.5em; margin: 0 0.5em;"/>
                    <Button on_click=move |_| navigate.clone()("/about", Default::default()) variant=ButtonVariant::Filled>
                        "About"
                    </Button>
                </Stack>
            </Stack>
        </AppBar>
    }
}
