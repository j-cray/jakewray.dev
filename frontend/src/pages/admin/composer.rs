use crate::components::rich_editor::RichTextEditor;
use leptos::prelude::*;

#[component]
pub fn AdminComposer() -> impl IntoView {
    let (content, set_content) = signal("<h1>New Post</h1><p>Start writing...</p>".to_string());

    let preview = move || content.get();

    view! {
        <div class="container py-12 h-screen flex flex-col">
            <div class="flex justify-between items-center mb-6">
                <h1 class="text-3xl font-bold">"Composer"</h1>
                <div class="flex gap-4">
                     <button class="btn btn-secondary">"Save Draft"</button>
                     <button class="btn btn-primary">"Publish"</button>
                </div>
            </div>

            <div class="flex-grow grid grid-cols-1 lg:grid-cols-2 gap-6 h-full">
                <div class="form-group h-full flex flex-col">
                    <label for="composer-content" class="font-bold mb-2">"Content"</label>
                    <RichTextEditor
                        id="composer-content".to_string()
                        value=content
                        on_change=move |new_val| set_content.set(new_val)
                        class="flex-grow min-h-[500px]"
                    />
                </div>

                <div class="card h-full flex flex-col p-4 border rounded-lg bg-gray-50">
                    <h3 class="text-lg font-bold mb-4">"Preview"</h3>
                    <div class="prose max-w-none overflow-y-auto flex-grow bg-white p-4 rounded border" inner_html=preview></div>
                </div>
            </div>
        </div>
    }
}
