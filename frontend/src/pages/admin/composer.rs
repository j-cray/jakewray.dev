use leptos::prelude::*;

#[component]
pub fn AdminComposer() -> impl IntoView {
    let (content, set_content) = signal("# New Post\n\nStart writing...".to_string());

    // Simple mock markdown parsing (replace newlines) for now.
    // TODO: Use proper markdown parser like pulldown-cmark or comrak
    // WARNING: This uses inner_html and is vulnerable to XSS. Only safe because
    // this is admin-only interface. Must use proper sanitization before production.
    let preview = move || {
        content
            .get()
            .replace("\n", "<br/>")
            .replace("# ", "<h1 class='text-2xl font-bold'>")
        // Very naive, just for scaffolding visual
    };

    view! {
        <div class="container py-12 h-screen flex flex-col">
            <div class="flex justify-between items-center mb-6">
                <h1 class="text-3xl">"Composer"</h1>
                <div class="flex gap-4">
                     <button class="px-4 py-2 border rounded-md">"Save Draft"</button>
                     <button class="px-4 py-2 bg-black text-white rounded-md">"Publish"</button>
                </div>
            </div>

            <div class="flex-grow grid grid-cols-2 gap-6 h-full">
                <textarea
                    class="w-full h-full p-4 border rounded-md font-mono resize-none focus:outline-none focus:ring-2 focus:ring-gray-200"
                    on:input=move |ev| set_content.set(event_target_value(&ev))
                    prop:value=content
                ></textarea>

                <div
                    class="w-full h-full p-4 border rounded-md bg-white prose overflow-y-auto"
                    inner_html=preview
                ></div>
            </div>
        </div>
    }
}
