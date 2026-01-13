use leptonic::prelude::*;

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer style="padding: 3em; margin-top: 3em; text-align: center;">
            <Stack orientation=StackOrientation::Vertical spacing=Size::Em(1.0)>
                <P>"© 2026 Jake Wray. All rights reserved."</P>
                <Stack orientation=StackOrientation::Horizontal spacing=Size::Em(1.0) style="justify-content: center;">
                    <Link href="/contact">"Contact"</Link>
                    <Link href="/admin">"Admin"</Link>
                </Stack>
            </Stack>
        </footer>
    }
}
