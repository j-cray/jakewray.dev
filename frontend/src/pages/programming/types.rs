use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PinnedRepo {
    pub name: String,
    pub owner: Option<String>,
    pub description: String,
    pub language: String,
    pub language_color: String,
    pub url: String,
    pub is_fork: bool,
    pub stars: Option<u32>,
    pub forks: Option<u32>,
    pub tags: Vec<String>,
}

impl PinnedRepo {
    pub fn display_name(&self) -> String {
        match &self.owner {
            Some(owner) => format!("{}/{}", owner, self.name),
            None => self.name.clone(),
        }
    }
}

pub fn get_pinned_repos() -> Vec<PinnedRepo> {
    vec![
        PinnedRepo {
            name: "jakewray.dev".to_string(),
            owner: None,
            description: "Personal portfolio and publishing platform built with Rust, Leptos SSR, Axum, and SQLite.".to_string(),
            language: "Rust".to_string(),
            language_color: "#dea584".to_string(),
            url: "https://github.com/j-cray/jakewray.dev".to_string(),
            is_fork: false,
            stars: None,
            forks: None,
            tags: vec!["rust".to_string(), "leptos".to_string(), "ssr".to_string(), "nix".to_string()],
        },
        PinnedRepo {
            name: "carapace".to_string(),
            owner: Some("puremachinery".to_string()),
            description: "Security-focused personal AI assistant with WASM sandboxing, secure-by-default architecture, and multi-channel messaging.".to_string(),
            language: "Rust".to_string(),
            language_color: "#dea584".to_string(),
            url: "https://github.com/puremachinery/carapace".to_string(),
            is_fork: false,
            stars: None,
            forks: None,
            tags: vec!["rust".to_string(), "ai-assistant".to_string(), "security".to_string(), "wasm".to_string()],
        },
        PinnedRepo {
            name: "NewsJournal".to_string(),
            owner: None,
            description: "Desktop application for journalists to organize story workflows, manage investigative beats, track sources, and monitor publishing deadlines.".to_string(),
            language: "Rust".to_string(),
            language_color: "#dea584".to_string(),
            url: "https://github.com/j-cray/NewsJournal".to_string(),
            is_fork: false,
            stars: None,
            forks: None,
            tags: vec!["rust".to_string(), "desktop".to_string(), "journalism".to_string(), "gui".to_string()],
        },
        PinnedRepo {
            name: "piotr".to_string(),
            owner: None,
            description: "A bro bot for Signal messaging written in Rust with SQLite storage, LLM integrations, and secure automation.".to_string(),
            language: "Rust".to_string(),
            language_color: "#dea584".to_string(),
            url: "https://github.com/j-cray/piotr".to_string(),
            is_fork: false,
            stars: None,
            forks: None,
            tags: vec!["rust".to_string(), "signal".to_string(), "bot".to_string(), "ai".to_string()],
        },
        PinnedRepo {
            name: "mastermind-obsidian".to_string(),
            owner: None,
            description: "Obsidian plugin integrating Google Vertex AI for context-aware assistance, note generation, and semantic vault search.".to_string(),
            language: "TypeScript".to_string(),
            language_color: "#3178c6".to_string(),
            url: "https://github.com/j-cray/mastermind-obsidian".to_string(),
            is_fork: false,
            stars: None,
            forks: None,
            tags: vec!["obsidian".to_string(), "typescript".to_string(), "vertex-ai".to_string(), "plugin".to_string()],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pinned_repos_not_empty() {
        let repos = get_pinned_repos();
        assert_eq!(repos.len(), 5);
        assert_eq!(repos[0].name, "jakewray.dev");
        assert_eq!(repos[0].display_name(), "jakewray.dev");
        assert_eq!(repos[1].name, "carapace");
        assert_eq!(repos[1].display_name(), "puremachinery/carapace");
    }

    #[test]
    fn test_pinned_repo_serde() {
        let repo = &get_pinned_repos()[0];
        let serialized = serde_json::to_string(repo).expect("Serialization failed");
        let deserialized: PinnedRepo =
            serde_json::from_str(&serialized).expect("Deserialization failed");
        assert_eq!(repo, &deserialized);
    }

    #[test]
    fn test_all_repos_have_valid_urls() {
        let repos = get_pinned_repos();
        for r in repos {
            assert!(r.url.starts_with("https://github.com/"));
            assert!(!r.description.is_empty());
            assert!(!r.language.is_empty());
            assert!(r.language_color.starts_with('#'));
        }
    }
}
