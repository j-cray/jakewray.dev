use leptos::prelude::*;
use web_sys::wasm_bindgen::JsCast;

fn execute_command(cmd: &str, value: Option<&str>) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(win) = web_sys::window() {
            if let Some(doc) = win.document() {
                if let Ok(html_doc) = doc.dyn_into::<web_sys::HtmlDocument>() {
                    let val = value.unwrap_or("");
                    let _ = html_doc.exec_command_with_show_ui_and_value(cmd, false, val);
                }
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (cmd, value);
    }
}

#[component]
pub fn RichTextEditor(
    #[prop(into)] value: Signal<String>,
    #[prop(into)] on_change: Callback<(String,)>,
    #[prop(into, optional)] id: Option<String>,
    #[prop(into, optional)] class: Option<String>,
) -> impl IntoView {
    let editor_id = id.unwrap_or_else(|| format!("editor-{}", uuid::Uuid::new_v4()));

    let update_html = move |target: Option<web_sys::EventTarget>| {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(target) = target {
                if let Ok(el) = target.dyn_into::<web_sys::HtmlElement>() {
                    let html = el.inner_html();
                    on_change.run((html,));
                }
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = target;
        }
    };

    let wrapper_class = format!(
        "rich-text-editor border rounded-lg overflow-hidden bg-white flex flex-col shadow-sm {}",
        class.unwrap_or_default()
    );

    let btn_class = "px-2.5 py-1.5 text-xs font-semibold rounded text-gray-700 hover:bg-gray-100 active:bg-gray-200 transition-colors flex items-center gap-1 border border-gray-200 bg-white";

    view! {
        <div class=wrapper_class>
            // Toolbar
            <div class="editor-toolbar flex flex-wrap gap-1.5 p-2 border-b bg-gray-50 items-center">
                // Formatting Group
                <div class="flex gap-1">
                    <button
                        type="button"
                        class=btn_class
                        title="Bold (Ctrl+B)"
                        on:mousedown=move |ev| ev.prevent_default()
                        on:click=move |_| execute_command("bold", None)
                    >
                        <span class="font-bold">"B"</span>
                    </button>

                    <button
                        type="button"
                        class=btn_class
                        title="Italic (Ctrl+I)"
                        on:mousedown=move |ev| ev.prevent_default()
                        on:click=move |_| execute_command("italic", None)
                    >
                        <span class="italic font-serif">"I"</span>
                    </button>

                    <button
                        type="button"
                        class=btn_class
                        title="Underline (Ctrl+U)"
                        on:mousedown=move |ev| ev.prevent_default()
                        on:click=move |_| execute_command("underline", None)
                    >
                        <span class="underline">"U"</span>
                    </button>

                    <button
                        type="button"
                        class=btn_class
                        title="Strikethrough"
                        on:mousedown=move |ev| ev.prevent_default()
                        on:click=move |_| execute_command("strikeThrough", None)
                    >
                        <span class="line-through">"S"</span>
                    </button>
                </div>

                <div class="h-4 w-px bg-gray-300 mx-0.5"></div>

                // Structure Group
                <div class="flex gap-1">
                    <button
                        type="button"
                        class=btn_class
                        title="Heading 1"
                        on:mousedown=move |ev| ev.prevent_default()
                        on:click=move |_| execute_command("formatBlock", Some("<h1>"))
                    >
                        <span class="font-bold">"H1"</span>
                    </button>

                    <button
                        type="button"
                        class=btn_class
                        title="Heading 2"
                        on:mousedown=move |ev| ev.prevent_default()
                        on:click=move |_| execute_command("formatBlock", Some("<h2>"))
                    >
                        <span class="font-bold">"H2"</span>
                    </button>

                    <button
                        type="button"
                        class=btn_class
                        title="Heading 3"
                        on:mousedown=move |ev| ev.prevent_default()
                        on:click=move |_| execute_command("formatBlock", Some("<h3>"))
                    >
                        <span class="font-bold">"H3"</span>
                    </button>

                    <button
                        type="button"
                        class=btn_class
                        title="Paragraph"
                        on:mousedown=move |ev| ev.prevent_default()
                        on:click=move |_| execute_command("formatBlock", Some("<p>"))
                    >
                        "P"
                    </button>
                </div>

                <div class="h-4 w-px bg-gray-300 mx-0.5"></div>

                // Lists & Quotes
                <div class="flex gap-1">
                    <button
                        type="button"
                        class=btn_class
                        title="Bullet List"
                        on:mousedown=move |ev| ev.prevent_default()
                        on:click=move |_| execute_command("insertUnorderedList", None)
                    >
                        "• List"
                    </button>

                    <button
                        type="button"
                        class=btn_class
                        title="Numbered List"
                        on:mousedown=move |ev| ev.prevent_default()
                        on:click=move |_| execute_command("insertOrderedList", None)
                    >
                        "1. List"
                    </button>

                    <button
                        type="button"
                        class=btn_class
                        title="Blockquote"
                        on:mousedown=move |ev| ev.prevent_default()
                        on:click=move |_| execute_command("formatBlock", Some("<blockquote>"))
                    >
                        "\" Quote"
                    </button>

                    <button
                        type="button"
                        class=btn_class
                        title="Horizontal Rule"
                        on:mousedown=move |ev| ev.prevent_default()
                        on:click=move |_| execute_command("insertHorizontalRule", None)
                    >
                        "— Line"
                    </button>
                </div>

                <div class="h-4 w-px bg-gray-300 mx-0.5"></div>

                // Alignment
                <div class="flex gap-1">
                    <button
                        type="button"
                        class=btn_class
                        title="Align Left"
                        on:mousedown=move |ev| ev.prevent_default()
                        on:click=move |_| execute_command("justifyLeft", None)
                    >
                        "Left"
                    </button>

                    <button
                        type="button"
                        class=btn_class
                        title="Align Center"
                        on:mousedown=move |ev| ev.prevent_default()
                        on:click=move |_| execute_command("justifyCenter", None)
                    >
                        "Center"
                    </button>

                    <button
                        type="button"
                        class=btn_class
                        title="Align Right"
                        on:mousedown=move |ev| ev.prevent_default()
                        on:click=move |_| execute_command("justifyRight", None)
                    >
                        "Right"
                    </button>
                </div>

                <div class="h-4 w-px bg-gray-300 mx-0.5"></div>

                // Links & Actions
                <div class="flex gap-1">
                    <button
                        type="button"
                        class=btn_class
                        title="Insert Link"
                        on:mousedown=move |ev| ev.prevent_default()
                        on:click=move |_| {
                            if let Ok(Some(url)) = web_sys::window().unwrap().prompt_with_message("Enter link URL:") {
                                if !url.trim().is_empty() {
                                    execute_command("createLink", Some(&url));
                                }
                            }
                        }
                    >
                        "🔗 Link"
                    </button>

                    <button
                        type="button"
                        class=btn_class
                        title="Remove Link"
                        on:mousedown=move |ev| ev.prevent_default()
                        on:click=move |_| execute_command("unlink", None)
                    >
                        "Unlink"
                    </button>

                    <button
                        type="button"
                        class=btn_class
                        title="Clear Formatting"
                        on:mousedown=move |ev| ev.prevent_default()
                        on:click=move |_| execute_command("removeFormat", None)
                    >
                        "Clear"
                    </button>
                </div>

                <div class="flex-grow"></div>

                // History
                <div class="flex gap-1">
                    <button
                        type="button"
                        class=btn_class
                        title="Undo (Ctrl+Z)"
                        on:mousedown=move |ev| ev.prevent_default()
                        on:click=move |_| execute_command("undo", None)
                    >
                        "↩"
                    </button>

                    <button
                        type="button"
                        class=btn_class
                        title="Redo (Ctrl+Y)"
                        on:mousedown=move |ev| ev.prevent_default()
                        on:click=move |_| execute_command("redo", None)
                    >
                        "↪"
                    </button>
                </div>
            </div>

            // Contenteditable Editor View
            <div
                id=editor_id
                class="editor-content p-6 min-h-[350px] max-h-[600px] overflow-y-auto prose max-w-none focus:outline-none bg-white text-black leading-relaxed"
                contenteditable="true"
                inner_html=value.get_untracked()
                on:input=move |ev| update_html(ev.target())
                on:keyup=move |ev| update_html(ev.target())
                on:blur=move |ev| update_html(ev.target())
            ></div>
        </div>
    }
}
