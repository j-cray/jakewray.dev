use icondata::BsList;
use leptos::prelude::*;
use leptos::*;

use leptonic::components::app_bar::AppBar;
use leptonic::components::button::{Button, ButtonVariant};
use leptonic::components::drawer::{Drawer, DrawerSide};
use leptonic::components::icon::Icon;
use leptonic::components::stack::{Stack, StackOrientation};
use leptonic::components::typography::H3;
use leptonic::prelude::Size;

#[component]
pub fn Navbar() -> impl IntoView {
    let navigate = leptos_router::hooks::use_navigate();
    let (is_drawer_open, set_drawer_open) = create_signal(false);

    let nav_btn = {
        let navigate = navigate.clone();
        move |text: &'static str, path: &'static str| {
            let navigate = navigate.clone();
            view! {
                <Button on_click=move |_| { navigate(path, Default::default()); set_drawer_open.set(false); } variant=ButtonVariant::Ghost style="font-weight: 500;">
                    {text}
                </Button>
            }
        }
    };

    view! {
        <AppBar style="z-index: 50; position: sticky; top: 0;" class="glass">
            <Stack orientation=StackOrientation::Horizontal spacing=Size::Em(1.0) style="width: 100%; justify-content: space-between; align-items: center; padding: 0 1em; height: 100%;">
                <H3 style="margin: 0; cursor: pointer;" on:click=move |_| navigate("/", Default::default())>
                    <span class="text-gradient" style="font-weight: bold;">"JAKE WRAY"</span>
                </H3>

                // Desktop Menu
                <Stack orientation=StackOrientation::Horizontal spacing=Size::Em(0.5) class="hidden md:flex">
                    {nav_btn("Journalism", "/journalism")}
                    {nav_btn("Personal", "/personal")}
                    {nav_btn("Writing", "/creative-writing")}
                    {nav_btn("Music", "/music")}
                    {nav_btn("Art", "/visual-art")}
                    {nav_btn("Code", "/programming")}

                    <Button on_click=move |_| navigate.clone()("/about", Default::default()) variant=ButtonVariant::Filled>
                        "About"
                    </Button>
                </Stack>

                // Mobile Menu Toggle
                <Button on_click=move |_| set_drawer_open.set(true) variant=ButtonVariant::Ghost class="md:hidden">
                    <Icon icon=icondata::BsList style="font-size: 1.5em;"/>
                </Button>
            </Stack>
        </AppBar>

        <Drawer side=DrawerSide::Right shown=is_drawer_open style="padding: 2em; background: var(--surface-color);">
            <Stack orientation=StackOrientation::Vertical spacing=Size::Em(1.5)>
                <H3>"Menu"</H3>
                {nav_btn("Journalism", "/journalism")}
                {nav_btn("Personal", "/personal")}
                {nav_btn("Writing", "/creative-writing")}
                {nav_btn("Music", "/music")}
                {nav_btn("Art", "/visual-art")}
                {nav_btn("Code", "/programming")}
                {nav_btn("About", "/about")}
                <Button on_click=move |_| set_drawer_open.set(false) variant=ButtonVariant::Outlined>
                    "Close"
                </Button>
            </Stack>
        </Drawer>
    }
}
