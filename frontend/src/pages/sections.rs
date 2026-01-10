use leptos::*;

#[component]
pub fn JournalismPage() -> impl IntoView {
    view! {
        <div class="container py-12">
            <h1 class="text-4xl mb-6">"Journalism"</h1>
             <div class="flex gap-4 mb-8">
                <button class="px-4 py-2 rounded-full border border-gray-300 hover:bg-gray-100">"All"</button>
                <button class="px-4 py-2 rounded-full border border-gray-300 hover:bg-gray-100">"Articles"</button>
                <button class="px-4 py-2 rounded-full border border-gray-300 hover:bg-gray-100">"Photojournalism"</button>
                <button class="px-4 py-2 rounded-full border border-gray-300 hover:bg-gray-100">"Video"</button>
                <button class="px-4 py-2 rounded-full border border-gray-300 hover:bg-gray-100">"J-School"</button>
            </div>
            <p class="text-muted">"Loading articles..."</p>
        </div>
    }
}

#[component]
pub fn PersonalPage() -> impl IntoView {
    view! {
        <div class="container py-12">
            <h1 class="text-4xl mb-6">"Personal"</h1>
            <p class="text-muted">"Blog, Photography, and Videography."</p>
        </div>
    }
}

#[component]
pub fn CreativeWritingPage() -> impl IntoView {
    view! {
        <div class="container py-12">
            <h1 class="text-4xl mb-6">"Creative Writing"</h1>
             <p class="text-muted">"Stories, Novels, and Poetry."</p>
        </div>
    }
}

#[component]
pub fn MusicPage() -> impl IntoView {
    view! {
        <div class="container py-12">
            <h1 class="text-4xl mb-6">"Music"</h1>
             <p class="text-muted">"Original compositions."</p>
        </div>
    }
}

#[component]
pub fn VisualArtPage() -> impl IntoView {
    view! {
        <div class="container py-12">
            <h1 class="text-4xl mb-6">"Visual Art"</h1>
             <p class="text-muted">"Drawings and Digital Art."</p>
        </div>
    }
}

#[component]
pub fn ProgrammingPage() -> impl IntoView {
    view! {
        <div class="container py-12">
             <h1 class="text-4xl mb-6">"Programming"</h1>
             <p class="text-muted">"GitHub Showcase."</p>
        </div>
    }
}
