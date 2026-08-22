pub fn strip_tags(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for ch in s.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ => {
                if !in_tag {
                    out.push(ch)
                }
            }
        }
    }
    out.trim().to_string()
}

pub fn starts_with_month(s: &str) -> bool {
    let sm = s.trim_start();
    const MONTHS: [&str; 21] = [
        "Jan.",
        "January",
        "Feb.",
        "February",
        "Mar.",
        "March",
        "Apr.",
        "April",
        "May",
        "June",
        "July",
        "Aug.",
        "August",
        "Sept.",
        "September",
        "Oct.",
        "October",
        "Nov.",
        "November",
        "Dec.",
        "December",
    ];
    MONTHS.iter().any(|m| {
        if let Some(after) = sm.strip_prefix(m) {
            // Match if it's the end of string or next char is not a letter
            after.chars().next().is_none_or(|c| !c.is_alphabetic())
        } else {
            false
        }
    })
}

pub fn extract_between(
    haystack: &str,
    start_pat: &str,
    end_pat: &str,
    from: usize,
) -> Option<(String, usize)> {
    let start_idx = haystack[from..].find(start_pat)? + from;
    let after = start_idx + start_pat.len();
    let end_idx = haystack[after..].find(end_pat)? + after;
    Some((
        haystack[after..end_idx].to_string(),
        end_idx + end_pat.len(),
    ))
}

#[allow(dead_code)]
pub fn extract_subhead(html: &str) -> Option<String> {
    let (inner, _) = extract_between(html, "<h4", "</h4>", 0)?;
    // drop attributes in opening tag
    let open_end = inner.find('>')? + 1;
    Some(strip_tags(&inner[open_end..]))
}

pub fn extract_printed_date(html: &str) -> Option<String> {
    let mut pos = 0;
    for _ in 0..5 {
        if let Some((p_inner, next)) = extract_between(html, "<p", "</p>", pos) {
            let open_end = p_inner.find('>').map(|i| i + 1).unwrap_or(0);
            let text = strip_tags(&p_inner[open_end..]);
            if starts_with_month(&text) {
                return Some(text);
            }
            pos = next;
        } else {
            break;
        }
    }
    None
}

pub fn extract_body_preview(html: &str) -> Option<String> {
    let mut pos = 0;
    for _ in 0..12 {
        let (p_inner, next) = extract_between(html, "<p", "</p>", pos)?;
        let open_end = p_inner.find('>').map(|i| i + 1).unwrap_or(0);
        let text = strip_tags(&p_inner[open_end..]);
        let t = text.trim();
        if !t.is_empty() && !starts_with_month(t) && !t.starts_with("By ") {
            return Some(t.to_string());
        }
        pos = next;
    }
    None
}

pub fn replace_date_paragraph(html: &str, new_date: &str) -> String {
    let mut pos = 0;
    for _ in 0..5 {
        if let Some((p_inner, next)) = extract_between(html, "<p", "</p>", pos) {
            let open_end = p_inner.find('>').map(|i| i + 1).unwrap_or(0);
            let text = strip_tags(&p_inner[open_end..]);
            if starts_with_month(&text) {
                if let Some(start_rel) = html[pos..].find("<p") {
                    let start_abs = pos + start_rel;
                    let after_start = start_abs + 2; // <p len
                    if let Some(end_rel) = html[after_start..].find("</p>") {
                        let end_abs = after_start + end_rel + 4; // </p> len
                        let mut out = html.to_string();
                        // Construct replacement paragraph
                        let formatted_date = format_cp_style(new_date);
                        let replacement = format!(
                            "<p class=\"text-sm text-gray-500 mb-6 mt-6\">{}</p>",
                            formatted_date
                        );
                        out.replace_range(start_abs..end_abs, &replacement);
                        return out;
                    }
                }
            }
            pos = next;
        } else {
            break;
        }
    }
    html.to_string()
}

pub fn bold_byline(html: &str) -> String {
    let mut out = html.to_string();
    let mut pos = 0;
    for _ in 0..10 {
        if let Some((p_inner, next)) = extract_between(&out, "<p", "</p>", pos) {
            let open_end = p_inner.find('>').map(|i| i + 1).unwrap_or(0);
            let text = strip_tags(&p_inner[open_end..]);
            let t = text.trim();
            if t.starts_with("By ") {
                if let Some(start_rel) = out[pos..].find("<p") {
                    let start_abs = pos + start_rel;
                    let after_start = start_abs + 2;
                    if let Some(end_rel) = out[after_start..].find("</p>") {
                        let end_abs = after_start + end_rel + 4;
                        let replacement = format!("<p><strong>{}</strong></p>", t);
                        out.replace_range(start_abs..end_abs, &replacement);
                        break;
                    }
                }
            }
            pos = next;
        } else {
            break;
        }
    }
    out
}

pub fn linkify_images(html: &str) -> String {
    let mut out = String::with_capacity(html.len() + 256);
    let mut pos = 0;

    while let Some(img_start_rel) = html[pos..].find("<img ") {
        let img_start = pos + img_start_rel;
        out.push_str(&html[pos..img_start]);

        // Check if already enclosed in an <a> tag immediately preceding (ignoring whitespace)
        let prefix = &html[..img_start];
        let trimmed_prefix = prefix.trim_end();
        let already_linked = trimmed_prefix.ends_with('>')
            && !trimmed_prefix.ends_with("</a>")
            && trimmed_prefix.rfind('<').is_some_and(|open_tag_idx| {
                let tag_content = &trimmed_prefix[open_tag_idx..];
                tag_content.starts_with("<a ") || tag_content.starts_with("<a>")
            });

        let img_end = if let Some(close_idx) = html[img_start..].find('>') {
            img_start + close_idx + 1
        } else {
            html.len()
        };

        let img_tag = &html[img_start..img_end];

        if already_linked {
            out.push_str(img_tag);
        } else {
            // Extract src attribute
            let mut src_val = None;
            if let Some(src_idx) = img_tag.find("src=\"") {
                let after_src = src_idx + 5;
                if let Some(end_quote) = img_tag[after_src..].find('"') {
                    src_val = Some(&img_tag[after_src..after_src + end_quote]);
                }
            } else if let Some(src_idx) = img_tag.find("src=\'") {
                let after_src = src_idx + 5;
                if let Some(end_quote) = img_tag[after_src..].find('\'') {
                    src_val = Some(&img_tag[after_src..after_src + end_quote]);
                }
            }

            if let Some(src) = src_val {
                out.push_str(&format!(
                    "<a href=\"{}\" target=\"_blank\" rel=\"noopener noreferrer\" class=\"article-image-link\">{}</a>",
                    src, img_tag
                ));
            } else {
                out.push_str(img_tag);
            }
        }

        pos = img_end;
    }

    out.push_str(&html[pos..]);
    out
}

pub fn italicize_origin_line(html: &str) -> String {
    let mut out = html.to_string();
    let mut pos = 0;
    for _ in 0..10 {
        if let Some((p_inner, next)) = extract_between(&out, "<p", "</p>", pos) {
            let open_end = p_inner.find('>').map(|i| i + 1).unwrap_or(0);
            let text = strip_tags(&p_inner[open_end..]);
            let t = text.trim();
            if t.starts_with("This article was originally published in")
                || t.starts_with("Originally published in")
            {
                if let Some(start_rel) = out[pos..].find("<p") {
                    let start_abs = pos + start_rel;
                    let after_start = start_abs + 2;
                    if let Some(end_rel) = out[after_start..].find("</p>") {
                        let end_abs = after_start + end_rel + 4;
                        let replacement = format!("<p><em>{}</em></p>", t);
                        out.replace_range(start_abs..end_abs, &replacement);
                        break;
                    }
                }
            }
            pos = next;
        } else {
            break;
        }
    }
    out
}

pub fn format_cp_style(date: &str) -> String {
    // Basic CP style formatter mapping standard month abbreviations to CP style
    date.replace("September", "Sept.")
        .replace("October", "Oct.")
        .replace("November", "Nov.")
        .replace("December", "Dec.")
        .replace("January", "Jan.")
        .replace("February", "Feb.")
        .replace("August", "Aug.")
}

pub fn extract_figcaption(content_html: &str) -> Option<String> {
    if let Some(start) = content_html.find("<figcaption") {
        if let Some(tag_end) = content_html[start..].find('>') {
            let content_start = start + tag_end + 1;
            if let Some(close_tag) = content_html[content_start..].find("</figcaption>") {
                let caption_text = content_html[content_start..content_start + close_tag].trim();
                if !caption_text.is_empty() {
                    return Some(caption_text.to_string());
                }
            }
        }
    }
    None
}

pub fn process_article_content(html: &str) -> String {
    let bolded = bold_byline(html);
    let linked = linkify_images(&bolded);
    italicize_origin_line(&linked)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_figcaption_valid() {
        let html =
            "<figure><img src=\"test.jpg\" /><figcaption>Test Caption Here</figcaption></figure>";
        assert_eq!(
            extract_figcaption(html),
            Some("Test Caption Here".to_string())
        );

        let html_attrs = "<figure><img src=\"test.jpg\" /><figcaption class=\"wp-caption-text\">Another Caption</figcaption></figure>";
        assert_eq!(
            extract_figcaption(html_attrs),
            Some("Another Caption".to_string())
        );
    }

    #[test]
    fn test_extract_figcaption_none() {
        let html = "<figure><img src=\"test.jpg\" /></figure>";
        assert_eq!(extract_figcaption(html), None);

        let html_empty = "<figure><img src=\"test.jpg\" /><figcaption>   </figcaption></figure>";
        assert_eq!(extract_figcaption(html_empty), None);
    }

    #[test]
    fn test_replace_date_paragraph() {
        let html = "<h4>Headline</h4><p>By Author</p><p>January 15, 2026</p><p>Article content here...</p>";
        let replaced = replace_date_paragraph(html, "Jan. 15, 2026");
        assert!(replaced.contains("<p class=\"text-sm text-gray-500 mb-6 mt-6\">Jan. 15, 2026</p>"));
        assert!(!replaced.contains("<p>January 15, 2026</p>"));
    }

    #[test]
    fn test_process_article_content_preserves_initial_heading() {
        let content = "<h4>A Subtitle Here</h4><p>By Jake Wray</p><p>Some actual text.</p>";
        let processed = process_article_content(content);
        assert!(processed.contains("<h4>A Subtitle Here</h4>"));
        assert!(processed.contains("<p><strong>By Jake Wray</strong></p>"));
    }

    #[test]
    fn test_process_article_content_transforms() {
        let content = "<p>By Jake Wray</p><p><img src=\"/images/photo.jpg\" alt=\"Test\"></p><p>This article was originally published in The Terrace Standard.</p>";
        let processed = process_article_content(content);
        assert!(processed.contains("<p><strong>By Jake Wray</strong></p>"));
        assert!(processed.contains("<a href=\"/images/photo.jpg\" target=\"_blank\" rel=\"noopener noreferrer\" class=\"article-image-link\"><img src=\"/images/photo.jpg\" alt=\"Test\"></a>"));
        assert!(processed.contains(
            "<p><em>This article was originally published in The Terrace Standard.</em></p>"
        ));
    }

    #[test]
    fn test_extract_body_preview_with_h4() {
        let html = "<h4>Subhead</h4><p>By Jake Wray</p><p>Jan. 15, 2026</p><p>First real sentence of the body.</p>";
        let preview = extract_body_preview(html);
        assert_eq!(
            preview,
            Some("First real sentence of the body.".to_string())
        );
    }
}
