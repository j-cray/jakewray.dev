#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use web_sys::window;

/// Determines whether a given route path belongs to the administrative section.
pub fn is_admin_path(path: &str) -> bool {
    path == "/admin" || path.starts_with("/admin/")
}

/// Checks whether analytics should be dispatched for the given route and user role.
/// Tracking is disabled for admin users and administrative routes.
pub fn should_track(path: &str, is_admin: bool) -> bool {
    !is_admin && !is_admin_path(path)
}

/// Dispatches a raw `gtag` invocation in a WASM/browser context.
/// If `gtag` is missing (e.g. adblock, non-production environment, or script blocked),
/// this function cleanly no-ops without throwing or panicking.
#[cfg(target_arch = "wasm32")]
fn call_gtag(args: &[JsValue]) {
    if let Some(win) = window() {
        if let Ok(gtag_val) = js_sys::Reflect::get(&win, &JsValue::from_str("gtag")) {
            if gtag_val.is_function() {
                let func: js_sys::Function = gtag_val.unchecked_into();
                let args_arr = js_sys::Array::new();
                for arg in args {
                    args_arr.push(arg);
                }
                let _ = func.apply(&win, &args_arr);
            }
        }
    }
}

/// Dispatches a GA4 `page_view` event for SPA route transitions.
pub fn track_page_view(path: &str, title: &str, is_admin: bool) {
    if !should_track(path, is_admin) {
        return;
    }

    #[cfg(target_arch = "wasm32")]
    {
        let params = js_sys::Object::new();
        let _ = js_sys::Reflect::set(&params, &"page_path".into(), &path.into());
        let _ = js_sys::Reflect::set(&params, &"page_title".into(), &title.into());

        if let Some(win) = window() {
            if let Ok(loc) = win.location().href() {
                let _ = js_sys::Reflect::set(&params, &"page_location".into(), &loc.into());
            }
        }

        call_gtag(&[
            JsValue::from_str("event"),
            JsValue::from_str("page_view"),
            params.into(),
        ]);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        tracing::debug!(path, title, "SSR/non-wasm: track_page_view");
    }
}

/// Dispatches a custom GA4 event with key-value string parameters.
pub fn track_event(event_name: &str, params: &[(&str, &str)], is_admin: bool) {
    if is_admin {
        return;
    }

    #[cfg(target_arch = "wasm32")]
    {
        let js_params = js_sys::Object::new();
        for (key, val) in params {
            let _ = js_sys::Reflect::set(&js_params, &(*key).into(), &(*val).into());
        }

        call_gtag(&[
            JsValue::from_str("event"),
            JsValue::from_str(event_name),
            js_params.into(),
        ]);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        tracing::debug!(event_name, ?params, "SSR/non-wasm: track_event");
    }
}

/// Helper to track clicks on outbound links (e.g. GitHub repos, external publications).
pub fn track_outbound_click(url: &str, label: Option<&str>, is_admin: bool) {
    let link_text = label.unwrap_or(url);
    track_event(
        "click",
        &[
            ("event_category", "outbound"),
            ("event_label", url),
            ("link_url", url),
            ("link_text", link_text),
        ],
        is_admin,
    );
}

/// Helper to track when an article/story is viewed.
pub fn track_article_view(slug: &str, title: &str, is_admin: bool) {
    track_event(
        "article_view",
        &[("article_slug", slug), ("article_title", title)],
        is_admin,
    );
}

/// Helper to track content sharing actions.
pub fn track_share(content_type: &str, item_id: &str, method: &str, is_admin: bool) {
    track_event(
        "share",
        &[
            ("content_type", content_type),
            ("item_id", item_id),
            ("method", method),
        ],
        is_admin,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_admin_path() {
        assert!(is_admin_path("/admin"));
        assert!(is_admin_path("/admin/"));
        assert!(is_admin_path("/admin/dashboard"));
        assert!(is_admin_path("/admin/compose"));
        assert!(is_admin_path("/admin/media"));
        assert!(is_admin_path("/admin/password-change"));

        assert!(!is_admin_path("/"));
        assert!(!is_admin_path("/code"));
        assert!(!is_admin_path("/journalism"));
        assert!(!is_admin_path("/journalism/some-slug"));
        assert!(!is_admin_path("/blog"));
        assert!(!is_admin_path("/about"));
        assert!(!is_admin_path("/administrator"));
    }

    #[test]
    fn test_should_track() {
        // Public routes for regular users should be tracked
        assert!(should_track("/", false));
        assert!(should_track("/code", false));
        assert!(should_track("/journalism", false));
        assert!(should_track("/journalism/article-slug", false));
        assert!(should_track("/blog", false));
        assert!(should_track("/about", false));

        // Admin paths for regular users should NOT be tracked
        assert!(!should_track("/admin", false));
        assert!(!should_track("/admin/dashboard", false));

        // Public routes for admin users should NOT be tracked
        assert!(!should_track("/", true));
        assert!(!should_track("/code", true));
        assert!(!should_track("/journalism", true));
        assert!(!should_track("/blog", true));
        assert!(!should_track("/about", true));

        // Admin paths for admin users should NOT be tracked
        assert!(!should_track("/admin", true));
        assert!(!should_track("/admin/dashboard", true));
    }

    #[test]
    fn test_analytics_functions_safe_execution() {
        // Ensure functions can be called safely without panic on non-wasm targets
        track_page_view("/", "Home", false);
        track_page_view("/admin/dashboard", "Admin", false);
        track_page_view("/code", "Code", true);

        track_event("test_event", &[("key", "value")], false);
        track_event("test_event_admin", &[("key", "value")], true);

        track_outbound_click("https://github.com/j-cray", Some("GitHub Profile"), false);
        track_outbound_click("https://github.com/j-cray", None, true);

        track_article_view("my-story", "My Story Title", false);
        track_article_view("my-story", "My Story Title", true);

        track_share("article", "my-story", "clipboard", false);
        track_share("article", "my-story", "clipboard", true);
    }
}
