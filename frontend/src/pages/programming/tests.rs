use super::types::{get_pinned_repos, PinnedRepo};

#[test]
fn test_pinned_repos_integrity() {
    let repos = get_pinned_repos();
    assert!(!repos.is_empty(), "Pinned repos should not be empty");

    // Validate repository names
    let names: Vec<&str> = repos.iter().map(|r| r.name.as_str()).collect();
    assert!(names.contains(&"jakewray.dev"));
    assert!(names.contains(&"carapace"));
    assert!(names.contains(&"NewsJournal"));
    assert!(names.contains(&"piotr"));
    assert!(names.contains(&"mastermind-obsidian"));
}

#[test]
fn test_pinned_repo_display_names() {
    let repo_without_owner = PinnedRepo {
        name: "test-repo".to_string(),
        owner: None,
        description: "Test description".to_string(),
        language: "Rust".to_string(),
        language_color: "#dea584".to_string(),
        url: "https://github.com/j-cray/test-repo".to_string(),
        is_fork: false,
        stars: None,
        forks: None,
        tags: vec!["rust".to_string()],
    };
    assert_eq!(repo_without_owner.display_name(), "test-repo");

    let repo_with_owner = PinnedRepo {
        name: "test-repo".to_string(),
        owner: Some("org-name".to_string()),
        description: "Test description".to_string(),
        language: "Rust".to_string(),
        language_color: "#dea584".to_string(),
        url: "https://github.com/org-name/test-repo".to_string(),
        is_fork: false,
        stars: None,
        forks: None,
        tags: vec!["rust".to_string()],
    };
    assert_eq!(repo_with_owner.display_name(), "org-name/test-repo");
}

#[test]
fn test_repo_tags_not_empty() {
    let repos = get_pinned_repos();
    for repo in repos {
        assert!(
            !repo.tags.is_empty(),
            "Tags should not be empty for {}",
            repo.name
        );
    }
}
