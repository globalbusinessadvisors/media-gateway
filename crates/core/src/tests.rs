//! Integration tests for the core module

#[cfg(test)]
mod integration_tests {
    use crate::models::content::CanonicalContent;
    use crate::models::user::UserProfile;
    use crate::models::search::SearchQuery;
    use crate::types::{ContentType, Platform};

    #[test]
    fn test_content_creation_and_serialization() {
        let content = CanonicalContent::new(
            ContentType::Movie,
            "The Shawshank Redemption".to_string(),
            1994,
        );

        assert_eq!(content.title, "The Shawshank Redemption");
        assert_eq!(content.release_year, 1994);

        // Test serialization
        let json = serde_json::to_string(&content).expect("Failed to serialize");
        assert!(json.contains("The Shawshank Redemption"));

        // Test deserialization
        let deserialized: CanonicalContent =
            serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(deserialized.title, content.title);
    }

    #[test]
    fn test_user_profile_creation() {
        let user = UserProfile::new(
            "test@example.com".to_string(),
            "Test User".to_string(),
            "US".to_string(),
        );

        assert_eq!(user.email, "test@example.com");
        assert!(user.is_active);
        assert!(!user.has_active_subscription(Platform::Netflix));
    }

    #[test]
    fn test_search_query_creation() {
        let query = SearchQuery::new("science fiction".to_string());
        
        assert_eq!(query.query, "science fiction");
        assert_eq!(query.limit, 20);
        assert!(!query.use_personalization);
    }
}
