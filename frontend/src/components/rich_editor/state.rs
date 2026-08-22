#[derive(Clone, Debug, PartialEq, Default)]
pub struct ActiveStates {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strike: bool,
    pub code: bool,
    pub bullet_list: bool,
    pub ordered_list: bool,
    pub justify_left: bool,
    pub justify_center: bool,
    pub justify_right: bool,
    pub block_tag: String,
}

pub fn normalize_block_tag(raw: &str) -> String {
    let cleaned = raw
        .trim()
        .trim_matches(|c| c == '<' || c == '>')
        .to_lowercase();
    match cleaned.as_str() {
        "h1" => "h1".to_string(),
        "h2" => "h2".to_string(),
        "h3" => "h3".to_string(),
        "blockquote" => "blockquote".to_string(),
        "pre" => "pre".to_string(),
        _ => "p".to_string(),
    }
}

pub fn query_active_states(_editor_id: &str) -> ActiveStates {
    #[cfg(target_arch = "wasm32")]
    {
        use web_sys::wasm_bindgen::JsCast;
        let mut states = ActiveStates::default();
        if let Some(win) = web_sys::window() {
            if let Some(doc) = win.document() {
                if let Ok(html_doc) = doc.dyn_into::<web_sys::HtmlDocument>() {
                    states.bold = html_doc.query_command_state("bold").unwrap_or(false);
                    states.italic = html_doc.query_command_state("italic").unwrap_or(false);
                    states.underline = html_doc.query_command_state("underline").unwrap_or(false);
                    states.strike = html_doc
                        .query_command_state("strikeThrough")
                        .unwrap_or(false);
                    states.bullet_list = html_doc
                        .query_command_state("insertUnorderedList")
                        .unwrap_or(false);
                    states.ordered_list = html_doc
                        .query_command_state("insertOrderedList")
                        .unwrap_or(false);
                    states.justify_left =
                        html_doc.query_command_state("justifyLeft").unwrap_or(false);
                    states.justify_center = html_doc
                        .query_command_state("justifyCenter")
                        .unwrap_or(false);
                    states.justify_right = html_doc
                        .query_command_state("justifyRight")
                        .unwrap_or(false);

                    let block = html_doc
                        .query_command_value("formatBlock")
                        .unwrap_or_default()
                        .to_lowercase();
                    states.block_tag = normalize_block_tag(&block);
                }
            }
        }
        states
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        ActiveStates::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_block_tag() {
        assert_eq!(normalize_block_tag("<h1>"), "h1");
        assert_eq!(normalize_block_tag("H2"), "h2");
        assert_eq!(normalize_block_tag("  <H3> "), "h3");
        assert_eq!(normalize_block_tag("blockquote"), "blockquote");
        assert_eq!(normalize_block_tag("pre"), "pre");
        assert_eq!(normalize_block_tag("div"), "p");
        assert_eq!(normalize_block_tag("p"), "p");
    }

    #[test]
    fn test_active_states_default() {
        let states = ActiveStates::default();
        assert!(!states.bold);
        assert!(!states.italic);
        assert_eq!(states.block_tag, "");
    }
}
