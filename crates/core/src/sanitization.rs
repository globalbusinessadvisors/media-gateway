//! Input sanitization utilities for preventing XSS and injection attacks

use ammonia::Builder;
use std::collections::HashSet;

/// Sanitize HTML content, removing all potentially dangerous elements
pub fn sanitize_html(input: &str) -> String {
    ammonia::clean(input)
}

/// Sanitize HTML with custom allowed tags
pub fn sanitize_html_with_tags(input: &str, allowed_tags: &[&str]) -> String {
    let tags: HashSet<&str> = allowed_tags.iter().copied().collect();
    Builder::default()
        .tags(tags)
        .clean(input)
        .to_string()
}

/// Sanitize search query - allow only safe characters
pub fn sanitize_search_query(input: &str) -> String {
    input
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '-' || *c == '_' || *c == '\'')
        .take(256)
        .collect::<String>()
        .trim()
        .to_string()
}

/// Sanitize user display name
pub fn sanitize_display_name(input: &str) -> String {
    input
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '-' || *c == '_' || *c == '.')
        .take(64)
        .collect::<String>()
        .trim()
        .to_string()
}

/// Sanitize a generic text field
pub fn sanitize_text(input: &str, max_length: usize) -> String {
    let cleaned = ammonia::clean(input);
    cleaned.chars().take(max_length).collect()
}

/// Check if input contains potential XSS payloads
pub fn contains_xss_patterns(input: &str) -> bool {
    let lower = input.to_lowercase();
    let patterns = [
        "<script",
        "javascript:",
        "onerror=",
        "onload=",
        "onclick=",
        "onmouseover=",
        "onfocus=",
        "onblur=",
        "<iframe",
        "<object",
        "<embed",
        "data:text/html",
        "vbscript:",
    ];
    patterns.iter().any(|p| lower.contains(p))
}

/// Escape special characters for safe JSON embedding
pub fn escape_json_string(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_html_removes_script() {
        let input = "<script>alert('xss')</script>Hello";
        let result = sanitize_html(input);
        assert!(!result.contains("<script>"));
        assert!(result.contains("Hello"));
    }

    #[test]
    fn test_sanitize_html_removes_onclick() {
        let input = "<div onclick=\"alert('xss')\">Click me</div>";
        let result = sanitize_html(input);
        assert!(!result.contains("onclick"));
    }

    #[test]
    fn test_sanitize_search_query() {
        let input = "action <script>alert('xss')</script> movies";
        let result = sanitize_search_query(input);
        assert_eq!(result, "action scriptalertxssscript movies");
    }

    #[test]
    fn test_sanitize_search_query_max_length() {
        let input = "a".repeat(500);
        let result = sanitize_search_query(&input);
        assert_eq!(result.len(), 256);
    }

    #[test]
    fn test_sanitize_display_name() {
        let input = "John <script>Doe</script>";
        let result = sanitize_display_name(input);
        assert!(!result.contains("<script>"));
    }

    #[test]
    fn test_contains_xss_patterns() {
        assert!(contains_xss_patterns("<script>alert('xss')</script>"));
        assert!(contains_xss_patterns("javascript:alert('xss')"));
        assert!(contains_xss_patterns("<img onerror='alert(1)'>"));
        assert!(!contains_xss_patterns("normal search query"));
    }

    #[test]
    fn test_escape_json_string() {
        let input = "Hello \"World\"\nNew line";
        let result = escape_json_string(input);
        assert_eq!(result, "Hello \\\"World\\\"\\nNew line");
    }
}
