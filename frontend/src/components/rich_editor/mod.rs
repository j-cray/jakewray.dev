pub mod commands;
pub mod state;

pub use commands::{do_cmd, do_input, execute_editor_command};
pub use state::{normalize_block_tag, query_active_states, ActiveStates};

use leptos::prelude::*;

#[component]
pub fn RichTextEditor(
    #[prop(into)] value: Signal<String>,
    #[prop(into)] on_change: Callback<(String,)>,
    #[prop(into, optional)] id: Option<String>,
    #[prop(into, optional)] class: Option<String>,
) -> impl IntoView {
    let editor_id = id.unwrap_or_else(|| format!("editor-{}", uuid::Uuid::new_v4()));
    let editor_id_store = StoredValue::new(editor_id.clone());
    let active_states = RwSignal::new(ActiveStates::default());
    let is_html_mode = RwSignal::new(false);

    let wrapper_class = format!(
        "rich-text-editor border rounded-lg overflow-hidden bg-white flex flex-col shadow-sm {}",
        class.unwrap_or_default()
    );

    view! {
        <div class=wrapper_class>
            // Modern Toolbar
            <div class="editor-toolbar flex flex-wrap gap-1.5 p-2 border-b bg-gray-50 items-center">
                // View Toggle (Visual vs HTML)
                <div class="editor-segmented-toggle">
                    <button
                        type="button"
                        class=move || if !is_html_mode.get() { "editor-toggle-btn is-active" } else { "editor-toggle-btn" }
                        title="Visual / Rich Text View"
                        on:click=move |_| is_html_mode.set(false)
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z"/><circle cx="12" cy="12" r="3"/></svg>
                        <span>"Visual"</span>
                    </button>
                    <button
                        type="button"
                        class=move || if is_html_mode.get() { "editor-toggle-btn is-active" } else { "editor-toggle-btn" }
                        title="HTML Code View"
                        on:click=move |_| is_html_mode.set(true)
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/></svg>
                        <span>"HTML"</span>
                    </button>
                </div>

                <div class="editor-divider"></div>

                // Block Format Select
                <select
                    class="editor-select"
                    disabled=move || is_html_mode.get()
                    prop:value=move || active_states.get().block_tag
                    on:change=move |ev| {
                        let val = event_target_value(&ev);
                        let block_spec = format!("<{}>", val);
                        do_cmd(&editor_id_store.get_value(), "formatBlock", Some(&block_spec), &on_change, active_states);
                    }
                >
                    <option value="p">"Paragraph"</option>
                    <option value="h1">"Heading 1"</option>
                    <option value="h2">"Heading 2"</option>
                    <option value="h3">"Heading 3"</option>
                    <option value="blockquote">"Quote"</option>
                    <option value="pre">"Code Block"</option>
                </select>

                <div class="editor-divider"></div>

                // Formatting Group
                <div class="editor-group">
                    <button
                        type="button"
                        disabled=move || is_html_mode.get()
                        class=move || if active_states.get().bold && !is_html_mode.get() { "editor-btn is-active" } else { "editor-btn" }
                        title="Bold (Ctrl+B)"
                        on:mousedown=move |ev| ev.prevent_default()
                        on:click=move |_| do_cmd(&editor_id_store.get_value(), "bold", None, &on_change, active_states)
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M6 4h8a4 4 0 0 1 4 4 4 4 0 0 1-4 4H6z"/><path d="M6 12h9a4 4 0 0 1 4 4 4 4 0 0 1-4 4H6z"/></svg>
                    </button>

                    <button
                        type="button"
                        disabled=move || is_html_mode.get()
                        class=move || if active_states.get().italic && !is_html_mode.get() { "editor-btn is-active" } else { "editor-btn" }
                        title="Italic (Ctrl+I)"
                        on:mousedown=move |ev| ev.prevent_default()
                        on:click=move |_| do_cmd(&editor_id_store.get_value(), "italic", None, &on_change, active_states)
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="19" y1="4" x2="10" y2="4"/><line x1="14" y1="20" x2="5" y2="20"/><line x1="15" y1="4" x2="9" y2="20"/></svg>
                    </button>

                    <button
                        type="button"
                        disabled=move || is_html_mode.get()
                        class=move || if active_states.get().underline && !is_html_mode.get() { "editor-btn is-active" } else { "editor-btn" }
                        title="Underline (Ctrl+U)"
                        on:mousedown=move |ev| ev.prevent_default()
                        on:click=move |_| do_cmd(&editor_id_store.get_value(), "underline", None, &on_change, active_states)
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M6 3v7a6 6 0 0 0 6 6 6 6 0 0 0 6-6V3"/><line x1="4" y1="21" x2="20" y2="21"/></svg>
                    </button>

                    <button
                        type="button"
                        disabled=move || is_html_mode.get()
                        class=move || if active_states.get().strike && !is_html_mode.get() { "editor-btn is-active" } else { "editor-btn" }
                        title="Strikethrough"
                        on:mousedown=move |ev| ev.prevent_default()
                        on:click=move |_| do_cmd(&editor_id_store.get_value(), "strikeThrough", None, &on_change, active_states)
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M16 4H9a3 3 0 0 0-2.83 4M14 12a4 4 0 0 1 0 8H6"/><line x1="4" y1="12" x2="20" y2="12"/></svg>
                    </button>

                    <button
                        type="button"
                        disabled=move || is_html_mode.get()
                        class=move || if active_states.get().code && !is_html_mode.get() { "editor-btn is-active" } else { "editor-btn" }
                        title="Inline Code"
                        on:mousedown=move |ev| ev.prevent_default()
                        on:click=move |_| do_cmd(&editor_id_store.get_value(), "insertHTML", Some("<code>code</code>"), &on_change, active_states)
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/></svg>
                    </button>
                </div>

                <div class="editor-divider"></div>

                // Lists & Quotes Group
                <div class="editor-group">
                    <button
                        type="button"
                        disabled=move || is_html_mode.get()
                        class=move || if active_states.get().bullet_list && !is_html_mode.get() { "editor-btn is-active" } else { "editor-btn" }
                        title="Bullet List"
                        on:mousedown=move |ev| ev.prevent_default()
                        on:click=move |_| do_cmd(&editor_id_store.get_value(), "insertUnorderedList", None, &on_change, active_states)
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="8" y1="6" x2="21" y2="6"/><line x1="8" y1="12" x2="21" y2="12"/><line x1="8" y1="18" x2="21" y2="18"/><line x1="3" y1="6" x2="3.01" y2="6"/><line x1="3" y1="12" x2="3.01" y2="12"/><line x1="3" y1="18" x2="3.01" y2="18"/></svg>
                    </button>

                    <button
                        type="button"
                        disabled=move || is_html_mode.get()
                        class=move || if active_states.get().ordered_list && !is_html_mode.get() { "editor-btn is-active" } else { "editor-btn" }
                        title="Numbered List"
                        on:mousedown=move |ev| ev.prevent_default()
                        on:click=move |_| do_cmd(&editor_id_store.get_value(), "insertOrderedList", None, &on_change, active_states)
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="10" y1="6" x2="21" y2="6"/><line x1="10" y1="12" x2="21" y2="12"/><line x1="10" y1="18" x2="21" y2="18"/><path d="M4 6h1v4"/><path d="M4 10h2"/><path d="M6 18H4c0-1 2-2 2-3s-1-1.5-2-1"/></svg>
                    </button>

                    <button
                        type="button"
                        disabled=move || is_html_mode.get()
                        class="editor-btn"
                        title="Blockquote"
                        on:mousedown=move |ev| ev.prevent_default()
                        on:click=move |_| do_cmd(&editor_id_store.get_value(), "formatBlock", Some("<blockquote>"), &on_change, active_states)
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 21c3 0 7-1 7-8V5c0-1.25-.756-2.017-2-2H4c-1.25 0-2 .75-2 2v6c0 1.25.75 2 2 2h3c0 4-4 6-4 6z"/><path d="M15 21c3 0 7-1 7-8V5c0-1.25-.757-2.017-2-2h-4c-1.25 0-2 .75-2 2v6c0 1.25.75 2 2 2h3c0 4-4 6-4 6z"/></svg>
                    </button>

                    <button
                        type="button"
                        disabled=move || is_html_mode.get()
                        class="editor-btn"
                        title="Horizontal Divider"
                        on:mousedown=move |ev| ev.prevent_default()
                        on:click=move |_| do_cmd(&editor_id_store.get_value(), "insertHorizontalRule", None, &on_change, active_states)
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="5" y1="12" x2="19" y2="12"/></svg>
                    </button>
                </div>

                <div class="editor-divider"></div>

                // Alignment Group
                <div class="editor-group">
                    <button
                        type="button"
                        disabled=move || is_html_mode.get()
                        class=move || if active_states.get().justify_left && !is_html_mode.get() { "editor-btn is-active" } else { "editor-btn" }
                        title="Align Left"
                        on:mousedown=move |ev| ev.prevent_default()
                        on:click=move |_| do_cmd(&editor_id_store.get_value(), "justifyLeft", None, &on_change, active_states)
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="17" y1="10" x2="3" y2="10"/><line x1="21" y1="6" x2="3" y2="6"/><line x1="21" y1="14" x2="3" y2="14"/><line x1="15" y1="18" x2="3" y2="18"/></svg>
                    </button>

                    <button
                        type="button"
                        disabled=move || is_html_mode.get()
                        class=move || if active_states.get().justify_center && !is_html_mode.get() { "editor-btn is-active" } else { "editor-btn" }
                        title="Align Center"
                        on:mousedown=move |ev| ev.prevent_default()
                        on:click=move |_| do_cmd(&editor_id_store.get_value(), "justifyCenter", None, &on_change, active_states)
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="10" x2="6" y2="10"/><line x1="21" y1="6" x2="3" y2="6"/><line x1="21" y1="14" x2="3" y2="14"/><line x1="18" y1="18" x2="6" y2="18"/></svg>
                    </button>

                    <button
                        type="button"
                        disabled=move || is_html_mode.get()
                        class=move || if active_states.get().justify_right && !is_html_mode.get() { "editor-btn is-active" } else { "editor-btn" }
                        title="Align Right"
                        on:mousedown=move |ev| ev.prevent_default()
                        on:click=move |_| do_cmd(&editor_id_store.get_value(), "justifyRight", None, &on_change, active_states)
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="21" y1="10" x2="7" y2="10"/><line x1="21" y1="6" x2="3" y2="6"/><line x1="21" y1="14" x2="3" y2="14"/><line x1="21" y1="18" x2="9" y2="18"/></svg>
                    </button>
                </div>

                <div class="editor-divider"></div>

                // Media & Links Group
                <div class="editor-group">
                    <button
                        type="button"
                        disabled=move || is_html_mode.get()
                        class="editor-btn"
                        title="Insert Link"
                        on:mousedown=move |ev| ev.prevent_default()
                        on:click=move |_| {
                            #[cfg(target_arch = "wasm32")]
                            {
                                if let Ok(Some(url)) = web_sys::window().unwrap().prompt_with_message_and_default("Enter link URL:", "https://") {
                                    if !url.trim().is_empty() {
                                        do_cmd(&editor_id_store.get_value(), "createLink", Some(&url), &on_change, active_states);
                                    }
                                }
                            }
                        }
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/></svg>
                    </button>

                    <button
                        type="button"
                        disabled=move || is_html_mode.get()
                        class="editor-btn"
                        title="Remove Link"
                        on:mousedown=move |ev| ev.prevent_default()
                        on:click=move |_| do_cmd(&editor_id_store.get_value(), "unlink", None, &on_change, active_states)
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m18.84 12.25 1.72-1.71a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="m5.16 11.75-1.72 1.71a5 5 0 0 0 7.07 7.07l1.72-1.71"/><line x1="2" y1="2" x2="22" y2="22"/></svg>
                    </button>

                    <button
                        type="button"
                        disabled=move || is_html_mode.get()
                        class="editor-btn"
                        title="Insert Image"
                        on:mousedown=move |ev| ev.prevent_default()
                        on:click=move |_| {
                            #[cfg(target_arch = "wasm32")]
                            {
                                if let Ok(Some(url)) = web_sys::window().unwrap().prompt_with_message_and_default("Enter image URL:", "https://") {
                                    if !url.trim().is_empty() {
                                        do_cmd(&editor_id_store.get_value(), "insertImage", Some(&url), &on_change, active_states);
                                    }
                                }
                            }
                        }
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/></svg>
                    </button>

                    <button
                        type="button"
                        disabled=move || is_html_mode.get()
                        class="editor-btn"
                        title="Clear Formatting"
                        on:mousedown=move |ev| ev.prevent_default()
                        on:click=move |_| do_cmd(&editor_id_store.get_value(), "removeFormat", None, &on_change, active_states)
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m7 21-4.3-4.3a1 1 0 0 1 0-1.4l9.6-9.6a1 1 0 0 1 1.4 0l4.3 4.3a1 1 0 0 1 0 1.4L8.4 21Z"/><path d="M22 21H7"/><path d="m5 11 9 9"/></svg>
                    </button>
                </div>

                <div class="flex-grow"></div>

                // History Group
                <div class="editor-group">
                    <button
                        type="button"
                        disabled=move || is_html_mode.get()
                        class="editor-btn"
                        title="Undo (Ctrl+Z)"
                        on:mousedown=move |ev| ev.prevent_default()
                        on:click=move |_| do_cmd(&editor_id_store.get_value(), "undo", None, &on_change, active_states)
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 7v6h6"/><path d="M21 17a9 9 0 0 0-9-9 9 9 0 0 0-6 2.3L3 13"/></svg>
                    </button>

                    <button
                        type="button"
                        disabled=move || is_html_mode.get()
                        class="editor-btn"
                        title="Redo (Ctrl+Y)"
                        on:mousedown=move |ev| ev.prevent_default()
                        on:click=move |_| do_cmd(&editor_id_store.get_value(), "redo", None, &on_change, active_states)
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 7v6h-6"/><path d="M3 17a9 9 0 0 1 9-9 9 9 0 0 1 6 2.3l3 2.7"/></svg>
                    </button>
                </div>
            </div>

            // Main Editor View Area (Visual vs HTML)
            {move || {
                if is_html_mode.get() {
                    view! {
                        <textarea
                            class="editor-code-area w-full min-h-[350px] max-h-[600px] overflow-y-auto"
                            prop:value=move || value.get()
                            on:input=move |ev| on_change.run((event_target_value(&ev),))
                        ></textarea>
                    }.into_any()
                } else {
                    view! {
                        <div
                            id=editor_id.clone()
                            class="editor-content p-6 min-h-[350px] max-h-[600px] overflow-y-auto prose max-w-none focus:outline-none bg-white text-black leading-relaxed"
                            contenteditable="true"
                            inner_html=value.get_untracked()
                            on:input=move |ev| do_input(&editor_id_store.get_value(), ev.target(), &on_change, active_states)
                            on:keyup=move |ev| do_input(&editor_id_store.get_value(), ev.target(), &on_change, active_states)
                            on:mouseup=move |ev| do_input(&editor_id_store.get_value(), ev.target(), &on_change, active_states)
                            on:blur=move |ev| do_input(&editor_id_store.get_value(), ev.target(), &on_change, active_states)
                            on:focus=move |_| do_cmd(&editor_id_store.get_value(), "defaultParagraphSeparator", Some("p"), &on_change, active_states)
                        ></div>
                    }.into_any()
                }
            }}
        </div>
    }
}
