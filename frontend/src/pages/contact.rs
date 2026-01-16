use leptos::prelude::*;

#[component]
pub fn ContactPage() -> impl IntoView {
    view! {
        <div class="container py-12 max-w-2xl">
            <h1 class="text-4xl mb-6">"Contact"</h1>
            <form class="flex flex-col gap-4">
                <input type="email" placeholder="Your Email" class="p-3 border rounded-md" />
                <textarea placeholder="Message" rows="5" class="p-3 border rounded-md"></textarea>
                <button type="submit" class="bg-black text-white p-3 rounded-md font-bold">"Send Message"</button>
            </form>
        </div>
    }
}
