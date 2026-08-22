pub fn sanitize_slug(slug: &str) -> String {
    slug.trim().to_string()
}

pub fn sanitize_page_slug(slug: &str) -> String {
    slug.trim()
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect()
}

#[cfg(feature = "ssr")]
pub fn parse_article_date(date_str: &str) -> (String, String, String) {
    let trimmed = date_str.trim();
    if trimmed.is_empty() {
        let now = chrono::Utc::now();
        let iso = now.format("%Y-%m-%d").to_string();
        let pub_at = now.format("%Y-%m-%dT%H:%M:%S.000Z").to_string();
        let day = now.format("%d").to_string().parse::<u32>().unwrap_or(1);
        let display = format!("{} {}, {}", now.format("%B"), day, now.format("%Y"));
        return (pub_at, iso, display);
    }

    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(trimmed) {
        use chrono::Datelike;
        let iso = dt.format("%Y-%m-%d").to_string();
        let pub_at = dt.format("%Y-%m-%dT%H:%M:%S.000Z").to_string();
        let display = format!("{} {}, {}", dt.format("%B"), dt.day(), dt.year());
        return (pub_at, iso, display);
    }

    let mut normalized = trimmed.to_string();
    if normalized.starts_with("Sept.") || normalized.starts_with("Sept ") {
        normalized = normalized.replacen("Sept", "Sep", 1);
    }
    for m in &[
        "Jan", "Feb", "Mar", "Apr", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ] {
        let pattern = format!("{}.", m);
        if normalized.starts_with(&pattern) {
            normalized = normalized.replacen(&pattern, m, 1);
            break;
        }
    }

    let formats = [
        "%Y-%m-%d",
        "%Y-%-m-%-d",
        "%Y-%m-%-d",
        "%Y-%-m-%d",
        "%B %d, %Y",
        "%B %-d, %Y",
        "%B %d %Y",
        "%B %-d %Y",
        "%b %d, %Y",
        "%b %-d, %Y",
        "%b %d %Y",
        "%b %-d %Y",
        "%d %B %Y",
        "%-d %B %Y",
        "%d %b %Y",
        "%-d %b %Y",
        "%m/%d/%Y",
        "%-m/%-d/%Y",
        "%Y/%m/%d",
        "%Y/%-m/%-d",
    ];

    for fmt in &formats {
        if let Ok(nd) = chrono::NaiveDate::parse_from_str(&normalized, fmt) {
            use chrono::Datelike;
            let iso = nd.format("%Y-%m-%d").to_string();
            let pub_at = format!("{}T00:00:00.000Z", iso);
            let display = format!("{} {}, {}", nd.format("%B"), nd.day(), nd.year());
            return (pub_at, iso, display);
        }
    }

    if trimmed.len() >= 10 && &trimmed[4..5] == "-" && &trimmed[7..8] == "-" {
        let iso = trimmed[..10].to_string();
        let pub_at = format!("{}T00:00:00.000Z", iso);
        return (pub_at, iso, trimmed.to_string());
    }

    (
        "1970-01-01T00:00:00.000Z".to_string(),
        "1970-01-01".to_string(),
        trimmed.to_string(),
    )
}

#[cfg(not(feature = "ssr"))]
pub fn parse_article_date(date_str: &str) -> (String, String, String) {
    let trimmed = date_str.trim();
    (
        "1970-01-01T00:00:00.000Z".to_string(),
        "1970-01-01".to_string(),
        trimmed.to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_slug_preserves_special_characters() {
        assert_eq!(
            sanitize_slug("terrace-mayor-slams-1979-cn-agreement-forcing-$182"),
            "terrace-mayor-slams-1979-cn-agreement-forcing-$182"
        );
        assert_eq!(
            sanitize_slug("construction-of-bc-hydro’s-north-coast-transmissio"),
            "construction-of-bc-hydro’s-north-coast-transmissio"
        );
        assert_eq!(sanitize_slug("  my-article-slug  "), "my-article-slug");
    }

    #[test]
    fn test_sanitize_page_slug() {
        assert_eq!(sanitize_page_slug("About"), "about");
        assert_eq!(sanitize_page_slug("  about-me  "), "about-me");
        assert_eq!(sanitize_page_slug("About Me Page!"), "about-me-page-");
    }

    #[test]
    #[cfg(feature = "ssr")]
    fn test_parse_article_date() {
        let (pub_at, iso, display) = parse_article_date("2025-05-21");
        assert_eq!(pub_at, "2025-05-21T00:00:00.000Z");
        assert_eq!(iso, "2025-05-21");
        assert_eq!(display, "May 21, 2025");

        let (pub_at, iso, display) = parse_article_date("May 21, 2025");
        assert_eq!(pub_at, "2025-05-21T00:00:00.000Z");
        assert_eq!(iso, "2025-05-21");
        assert_eq!(display, "May 21, 2025");

        let (pub_at, iso, display) = parse_article_date("Jan. 15, 2024");
        assert_eq!(pub_at, "2024-01-15T00:00:00.000Z");
        assert_eq!(iso, "2024-01-15");
        assert_eq!(display, "January 15, 2024");

        let (pub_at, iso, display) = parse_article_date("2025-05-21T00:00:00.000Z");
        assert_eq!(pub_at, "2025-05-21T00:00:00.000Z");
        assert_eq!(iso, "2025-05-21");
        assert_eq!(display, "May 21, 2025");

        let (pub_at, iso, display) = parse_article_date("2026-8-5");
        assert_eq!(pub_at, "2026-08-05T00:00:00.000Z");
        assert_eq!(iso, "2026-08-05");
        assert_eq!(display, "August 5, 2026");

        let (pub_at, iso, display) = parse_article_date("8/5/2026");
        assert_eq!(pub_at, "2026-08-05T00:00:00.000Z");
        assert_eq!(iso, "2026-08-05");
        assert_eq!(display, "August 5, 2026");
    }
}
