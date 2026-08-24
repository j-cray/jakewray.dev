use leptos::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdminActionIcon {
    Edit,
    Compose,
    Dashboard,
    Media,
    Close,
}

#[derive(Clone, Debug)]
pub struct AdminAction {
    pub label: String,
    pub icon: AdminActionIcon,
    pub href: Option<String>,
    pub on_click: Option<Callback<()>>,
    pub is_active: bool,
}

impl PartialEq for AdminAction {
    fn eq(&self, other: &Self) -> bool {
        self.label == other.label
            && self.icon == other.icon
            && self.href == other.href
            && self.is_active == other.is_active
            && self.on_click.is_some() == other.on_click.is_some()
    }
}

#[derive(Clone, Copy)]
pub struct AdminContext {
    pub is_admin: ReadSignal<bool>,
    pub set_is_admin: WriteSignal<bool>,
    pub token: ReadSignal<String>,
    pub set_token: WriteSignal<String>,
    pub contextual_action: RwSignal<Option<AdminAction>>,
}

impl AdminContext {
    pub fn new() -> Self {
        let (is_admin, set_is_admin) = signal(false);
        let (token, set_token) = signal(String::new());
        let contextual_action = RwSignal::new(None);

        Self {
            is_admin,
            set_is_admin,
            token,
            set_token,
            contextual_action,
        }
    }

    pub fn login(&self, token_str: String) {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    let _ = storage.set_item("admin_token", &token_str);
                }
            }
        }
        self.set_token.set(token_str);
        self.set_is_admin.set(true);
    }

    pub fn logout(&self) {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    let _ = storage.remove_item("admin_token");
                }
            }
        }
        self.set_token.set(String::new());
        self.set_is_admin.set(false);
        self.contextual_action.set(None);
    }

    pub fn init_from_storage(&self) {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    if let Ok(Some(t)) = storage.get_item("admin_token") {
                        if !t.is_empty() {
                            if shared::auth::is_token_expired(&t) {
                                let _ = storage.remove_item("admin_token");
                                self.set_is_admin.set(false);
                                self.set_token.set(String::new());
                            } else {
                                self.set_token.set(t);
                                self.set_is_admin.set(true);
                            }
                            return;
                        }
                    }
                }
            }
            self.set_is_admin.set(false);
            self.set_token.set(String::new());
        }
    }

    pub fn set_action(&self, action: AdminAction) {
        self.contextual_action.set(Some(action));
    }

    pub fn clear_action(&self) {
        self.contextual_action.set(None);
    }
}

impl Default for AdminContext {
    fn default() -> Self {
        Self::new()
    }
}

pub fn provide_admin_context() -> AdminContext {
    let ctx = AdminContext::new();
    provide_context(ctx);
    ctx
}

pub fn use_admin_context() -> AdminContext {
    use_context::<AdminContext>().unwrap_or_else(|| {
        #[cfg(debug_assertions)]
        leptos::logging::warn!("AdminContext not found in component tree, fallback instantiated");
        AdminContext::new()
    })
}

/// Fallback contextual action resolver when no custom page action has been dynamically registered.
pub fn get_default_contextual_action(pathname: &str) -> Option<AdminAction> {
    if pathname.starts_with("/blog") {
        Some(AdminAction {
            label: "Compose Blog Post".to_string(),
            icon: AdminActionIcon::Compose,
            href: Some("/admin/compose".to_string()),
            on_click: None,
            is_active: false,
        })
    } else if pathname == "/admin/dashboard" {
        Some(AdminAction {
            label: "Compose Post".to_string(),
            icon: AdminActionIcon::Compose,
            href: Some("/admin/compose".to_string()),
            on_click: None,
            is_active: false,
        })
    } else if pathname == "/admin/compose" {
        Some(AdminAction {
            label: "Admin Dashboard".to_string(),
            icon: AdminActionIcon::Dashboard,
            href: Some("/admin/dashboard".to_string()),
            on_click: None,
            is_active: false,
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_contextual_action_blog() {
        let action = get_default_contextual_action("/blog").expect("Expected action for /blog");
        assert_eq!(action.label, "Compose Blog Post");
        assert_eq!(action.icon, AdminActionIcon::Compose);
        assert_eq!(action.href, Some("/admin/compose".to_string()));
    }

    #[test]
    fn test_default_contextual_action_about() {
        let action = get_default_contextual_action("/about");
        assert_eq!(action, None);
    }

    #[test]
    fn test_default_contextual_action_article() {
        let action = get_default_contextual_action("/journalism/city-council-2026");
        assert_eq!(action, None);
    }

    #[test]
    fn test_default_contextual_action_journalism_list() {
        let action = get_default_contextual_action("/journalism");
        assert_eq!(action, None);

        let action_trailing = get_default_contextual_action("/journalism/");
        assert_eq!(action_trailing, None);
    }

    #[test]
    fn test_default_contextual_action_code_and_home() {
        let home_action = get_default_contextual_action("/");
        assert_eq!(home_action, None);

        let code_action = get_default_contextual_action("/code");
        assert_eq!(code_action, None);
    }

    #[test]
    fn test_admin_action_equality() {
        let a1 = AdminAction {
            label: "Test".to_string(),
            icon: AdminActionIcon::Edit,
            href: None,
            on_click: None,
            is_active: false,
        };
        let a2 = AdminAction {
            label: "Test".to_string(),
            icon: AdminActionIcon::Edit,
            href: None,
            on_click: None,
            is_active: false,
        };
        assert_eq!(a1, a2);
    }
}
