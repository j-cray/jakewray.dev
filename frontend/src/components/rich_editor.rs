use leptos::prelude::*;
use leptos_tiptap::*;

#[component]
pub fn RichTextEditor(
    #[prop(into)] value: Signal<String>,
    #[prop(into)] on_change: Callback<(String,)>,
    #[prop(into, optional)] id: Option<String>,
    #[prop(into, optional)] class: Option<String>,
) -> impl IntoView {
    let editor_id = id.unwrap_or_else(|| format!("tiptap-{}", uuid::Uuid::new_v4()));

    let (msg, set_msg) = signal(TiptapInstanceMsg::Noop);
    let (selection_state, set_selection_state) = signal(TiptapSelectionState::default());

    let set_value_callback = Callback::new(move |(content,): (TiptapContent,)| match content {
        TiptapContent::Html(html) => {
            on_change.run((html,));
        }
        TiptapContent::Json(json) => {
            on_change.run((json,));
        }
    });

    let on_selection_change_callback = Callback::new(move |(state,): (TiptapSelectionState,)| {
        set_selection_state.set(state);
    });

    let send_msg = move |m: TiptapInstanceMsg| {
        set_msg.set(m);
    };

    let wrapper_class = format!(
        "rich-text-editor border rounded-lg overflow-hidden bg-white flex flex-col {}",
        class.unwrap_or_default()
    );

    let is_bold = move || selection_state.get().bold;
    let is_italic = move || selection_state.get().italic;
    let is_strike = move || selection_state.get().strike;
    let is_h1 = move || selection_state.get().h1;
    let is_h2 = move || selection_state.get().h2;
    let is_h3 = move || selection_state.get().h3;
    let is_bullet = move || selection_state.get().bullet_list;
    let is_ordered = move || selection_state.get().ordered_list;
    let is_quote = move || selection_state.get().blockquote;
    let is_link = move || selection_state.get().link;

    let btn_class = |active: bool| -> &'static str {
        if active {
            "p-2 text-sm font-semibold rounded bg-blue-100 text-blue-700 hover:bg-blue-200 transition-colors"
        } else {
            "p-2 text-sm font-semibold rounded text-gray-700 hover:bg-gray-100 transition-colors"
        }
    };

    view! {
        <div class=wrapper_class>
            // Toolbar
            <div class="editor-toolbar flex flex-wrap gap-1 p-2 border-b bg-gray-50 items-center">
                <button
                    type="button"
                    class=move || btn_class(is_bold())
                    title="Bold"
                    on:click=move |_| send_msg(TiptapInstanceMsg::Bold)
                >
                    <span class="font-bold">"B"</span>
                </button>

                <button
                    type="button"
                    class=move || btn_class(is_italic())
                    title="Italic"
                    on:click=move |_| send_msg(TiptapInstanceMsg::Italic)
                >
                    <span class="italic">"I"</span>
                </button>

                <button
                    type="button"
                    class=move || btn_class(is_strike())
                    title="Strikethrough"
                    on:click=move |_| send_msg(TiptapInstanceMsg::Strike)
                >
                    <span class="line-through">"S"</span>
                </button>

                <div class="h-5 w-px bg-gray-300 mx-1"></div>

                <button
                    type="button"
                    class=move || btn_class(is_h1())
                    title="Heading 1"
                    on:click=move |_| send_msg(TiptapInstanceMsg::H1)
                >
                    "H1"
                </button>

                <button
                    type="button"
                    class=move || btn_class(is_h2())
                    title="Heading 2"
                    on:click=move |_| send_msg(TiptapInstanceMsg::H2)
                >
                    "H2"
                </button>

                <button
                    type="button"
                    class=move || btn_class(is_h3())
                    title="Heading 3"
                    on:click=move |_| send_msg(TiptapInstanceMsg::H3)
                >
                    "H3"
                </button>

                <button
                    type="button"
                    class=move || btn_class(!is_h1() && !is_h2() && !is_h3())
                    title="Paragraph"
                    on:click=move |_| send_msg(TiptapInstanceMsg::Paragraph)
                >
                    "P"
                </button>

                <div class="h-5 w-px bg-gray-300 mx-1"></div>

                <button
                    type="button"
                    class=move || btn_class(is_bullet())
                    title="Bullet List"
                    on:click=move |_| send_msg(TiptapInstanceMsg::BulletList)
                >
                    "• List"
                </button>

                <button
                    type="button"
                    class=move || btn_class(is_ordered())
                    title="Numbered List"
                    on:click=move |_| send_msg(TiptapInstanceMsg::OrderedList)
                >
                    "1. List"
                </button>

                <button
                    type="button"
                    class=move || btn_class(is_quote())
                    title="Blockquote"
                    on:click=move |_| send_msg(TiptapInstanceMsg::Blockquote)
                >
                    "\" Quote"
                </button>

                <div class="h-5 w-px bg-gray-300 mx-1"></div>

                <button
                    type="button"
                    class=move || btn_class(is_link())
                    title="Insert Link"
                    on:click=move |_| {
                        if is_link() {
                            send_msg(TiptapInstanceMsg::UnsetLink());
                        } else if let Ok(Some(url)) = web_sys::window().unwrap().prompt_with_message("Enter URL:") {
                            if !url.trim().is_empty() {
                                send_msg(TiptapInstanceMsg::SetLink(TiptapLinkResource {
                                    href: url,
                                    target: "_blank".to_string(),
                                    rel: "".to_string(),
                                }));
                            }
                        }
                    }
                >
                    "🔗 Link"
                </button>
            </div>

            // Editor Content Container
            <div class="editor-content p-4 min-h-[350px] max-h-[600px] overflow-y-auto prose max-w-none focus:outline-none">
                <TiptapInstance
                    id=editor_id
                    value=value
                    set_value=set_value_callback
                    msg=msg
                    disabled=false
                    on_selection_change=on_selection_change_callback
                />
            </div>
        </div>
    }
}
