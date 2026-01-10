use leptos::*; use leptos::prelude::*;

#[component]
pub fn Navbar() -> impl IntoView {
    view! {
        <nav class="sticky top-0 z-50 p-4 backdrop-blur-md bg-white/80 border-b border-gray-200">
            <div class="container flex flex-col md:flex-row justify-between items-center gap-4">
                <a href="/" class="text-2xl font-bold font-heading">"Jake Wray"</a>

                // Socials
                <div class="flex gap-4 text-xl">
                     <a href="#" aria-label="Bluesky"><i class="icon-bluesky"></i>"Bsky"</a>
                     // Icons would ideally be SVG components or a font. Using Text for now.
                </div>

                <ul class="flex flex-wrap gap-6 text-sm font-medium text-gray-600 justify-center">
                    <li><a href="/journalism" class="hover:text-black transition">"Journalism"</a></li>
                    <li><a href="/personal" class="hover:text-black transition">"Personal"</a></li>
                    <li><a href="/creative-writing" class="hover:text-black transition">"Writing"</a></li>
                    <li><a href="/music" class="hover:text-black transition">"Music"</a></li>
                    <li><a href="/visual-art" class="hover:text-black transition">"Art"</a></li>
                    <li><a href="/programming" class="hover:text-black transition">"Code"</a></li>
                    <li><a href="/about" class="hover:text-black transition">"About"</a></li>
                </ul>
            </div>
        </nav>
    }
}
