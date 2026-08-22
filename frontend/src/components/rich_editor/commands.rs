use crate::components::rich_editor::state::{query_active_states, ActiveStates};
use leptos::prelude::*;
#[cfg(target_arch = "wasm32")]
use web_sys::wasm_bindgen::JsCast;

pub fn execute_editor_command(
    editor_id: &str,
    cmd: &str,
    value: Option<&str>,
    on_change: &Callback<(String,)>,
) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(win) = web_sys::window() {
            if let Some(doc) = win.document() {
                if let Some(el) = doc.get_element_by_id(editor_id) {
                    if let Ok(html_el) = el.dyn_into::<web_sys::HtmlElement>() {
                        let _ = html_el.focus();
                    }
                }
                if let Ok(html_doc) = doc.clone().dyn_into::<web_sys::HtmlDocument>() {
                    let val = value.unwrap_or("");
                    let _ = html_doc.exec_command_with_show_ui_and_value(cmd, false, val);
                }
                if let Some(el) = doc.get_element_by_id(editor_id) {
                    if let Ok(html_el) = el.dyn_into::<web_sys::HtmlElement>() {
                        let html = html_el.inner_html();
                        on_change.run((html,));
                    }
                }
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (editor_id, cmd, value, on_change);
    }
}

pub fn do_cmd(
    editor_id: &str,
    cmd: &str,
    val: Option<&str>,
    on_change: &Callback<(String,)>,
    active_states: RwSignal<ActiveStates>,
) {
    execute_editor_command(editor_id, cmd, val, on_change);
    let current = query_active_states(editor_id);
    active_states.set(current);
}

pub fn do_input(
    editor_id: &str,
    target: Option<web_sys::EventTarget>,
    on_change: &Callback<(String,)>,
    active_states: RwSignal<ActiveStates>,
) {
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
        let _ = (target, on_change);
    }
    let current = query_active_states(editor_id);
    active_states.set(current);
}
