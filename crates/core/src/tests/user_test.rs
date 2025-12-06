//! User model tests

use crate::models::user::*;
use crate::types::*;
use chrono::{Duration, Utc};
use uuid::Uuid;

#[tokio::test]
async fn test_user_profile_creation_with_valid_data() {
    let email = "alice@example.com".to_string();
    let name = "Alice Smith".to_string();

    let profile = UserProfile::new(email.clone(), name.clone());

    assert_eq!(profile.email, email);
    assert_eq!(profile.display_name, name);
    assert!(profile.avatar_url.is_none());
    assert!(profile.is_active_user());
}

#[tokio::test]
async fn test_user_profile_has_unique_id() {
    let profile1 = UserProfile::new("user1@example.com".to_string(), "User 1".to_string());
    let profile2 = UserProfile::new("user2@example.com".to_string(), "User 2".to_string());

    assert_ne!(profile1.id, profile2.id);
}

#[tokio::test]
async fn test_user_preferences_add_favorite_genre_once() {
    let profile = UserProfile::new("test@example.com".to_string(), "Test".to_string());
    let mut prefs = profile.preferences.lock().await;

    prefs.add_favorite_genre(Genre::Action);
    prefs.add_favorite_genre(Genre::Drama);
    prefs.add_favorite_genre(Genre::SciFi);

    assert_eq!(prefs.favorite_genres.len(), 3);
    assert!(prefs.favorite_genres.contains(&Genre::Action));
    assert!(prefs.favorite_genres.contains(&Genre::Drama));
    assert!(prefs.favorite_genres.contains(&Genre::SciFi));
}

#[tokio::test]
async fn test_user_preferences_no_duplicate_favorite_genres() {
    let profile = UserProfile::new("test@example.com".to_string(), "Test".to_string());
    let mut prefs = profile.preferences.lock().await;

    prefs.add_favorite_genre(Genre::Action);
    prefs.add_favorite_genre(Genre::Action);
    prefs.add_favorite_genre(Genre::Action);

    assert_eq!(prefs.favorite_genres.len(), 1);
}

#[tokio::test]
async fn test_user_preferences_remove_favorite_genre() {
    let profile = UserProfile::new("test@example.com".to_string(), "Test".to_string());
    let mut prefs = profile.preferences.lock().await;

    prefs.add_favorite_genre(Genre::Action);
    prefs.add_favorite_genre(Genre::Drama);
    prefs.add_favorite_genre(Genre::Comedy);

    prefs.remove_favorite_genre(&Genre::Drama);

    assert_eq!(prefs.favorite_genres.len(), 2);
    assert!(prefs.favorite_genres.contains(&Genre::Action));
    assert!(!prefs.favorite_genres.contains(&Genre::Drama));
    assert!(prefs.favorite_genres.contains(&Genre::Comedy));
}

#[tokio::test]
async fn test_user_preferences_remove_nonexistent_genre() {
    let profile = UserProfile::new("test@example.com".to_string(), "Test".to_string());
    let mut prefs = profile.preferences.lock().await;

    prefs.add_favorite_genre(Genre::Action);
    prefs.remove_favorite_genre(&Genre::Horror); // Not in list

    assert_eq!(prefs.favorite_genres.len(), 1);
    assert!(prefs.favorite_genres.contains(&Genre::Action));
}

#[tokio::test]
async fn test_user_preferences_mutex_thread_safety() {
    let profile = UserProfile::new("test@example.com".to_string(), "Test".to_string());

    // Simulate concurrent access
    let prefs1 = profile.preferences.clone();
    let prefs2 = profile.preferences.clone();

    let task1 = tokio::spawn(async move {
        let mut p = prefs1.lock().await;
        p.add_favorite_genre(Genre::Action);
    });

    let task2 = tokio::spawn(async move {
        let mut p = prefs2.lock().await;
        p.add_favorite_genre(Genre::Drama);
    });

    task1.await.unwrap();
    task2.await.unwrap();

    let prefs = profile.preferences.lock().await;
    assert_eq!(prefs.favorite_genres.len(), 2);
}

#[test]
fn test_is_active_user_with_recent_activity() {
    let mut profile = UserProfile::new("test@example.com".to_string(), "Test".to_string());

    profile.last_active_at = Utc::now() - Duration::days(1);
    assert!(profile.is_active_user());

    profile.last_active_at = Utc::now() - Duration::days(15);
    assert!(profile.is_active_user());

    profile.last_active_at = Utc::now() - Duration::days(29);
    assert!(profile.is_active_user());
}

#[test]
fn test_is_active_user_with_old_activity() {
    let mut profile = UserProfile::new("test@example.com".to_string(), "Test".to_string());

    profile.last_active_at = Utc::now() - Duration::days(31);
    assert!(!profile.is_active_user());

    profile.last_active_at = Utc::now() - Duration::days(60);
    assert!(!profile.is_active_user());

    profile.last_active_at = Utc::now() - Duration::days(365);
    assert!(!profile.is_active_user());
}

#[test]
fn test_is_active_user_exactly_30_days_boundary() {
    let mut profile = UserProfile::new("test@example.com".to_string(), "Test".to_string());

    // Exactly 30 days ago should be considered inactive
    profile.last_active_at = Utc::now() - Duration::days(30);
    assert!(!profile.is_active_user());

    // Just under 30 days should be active
    profile.last_active_at = Utc::now() - Duration::days(30) + Duration::hours(1);
    assert!(profile.is_active_user());
}

#[test]
fn test_user_preferences_default_values() {
    let prefs = UserPreferences::default();

    assert!(prefs.favorite_genres.is_empty());
    assert!(prefs.blocked_maturity_ratings.is_empty());
    assert!(prefs.preferred_platforms.is_empty());
    assert_eq!(prefs.preferred_languages, vec!["en".to_string()]);
    assert!(prefs.autoplay_enabled);
    assert!(prefs.hd_preferred);
}

#[tokio::test]
async fn test_user_preferences_platform_preferences() {
    let profile = UserProfile::new("test@example.com".to_string(), "Test".to_string());
    let mut prefs = profile.preferences.lock().await;

    prefs.preferred_platforms.push(Platform::Netflix);
    prefs.preferred_platforms.push(Platform::HBOMax);
    prefs.preferred_platforms.push(Platform::DisneyPlus);

    assert_eq!(prefs.preferred_platforms.len(), 3);
}

#[tokio::test]
async fn test_user_preferences_maturity_rating_blocks() {
    let profile = UserProfile::new("test@example.com".to_string(), "Test".to_string());
    let mut prefs = profile.preferences.lock().await;

    prefs.blocked_maturity_ratings.push(MaturityRating::R);
    prefs.blocked_maturity_ratings.push(MaturityRating::NC17);
    prefs.blocked_maturity_ratings.push(MaturityRating::TVMA);

    assert_eq!(prefs.blocked_maturity_ratings.len(), 3);
    assert!(prefs.blocked_maturity_ratings.contains(&MaturityRating::R));
}

#[tokio::test]
async fn test_user_preferences_language_preferences() {
    let profile = UserProfile::new("test@example.com".to_string(), "Test".to_string());
    let mut prefs = profile.preferences.lock().await;

    prefs.preferred_languages = vec!["en".to_string(), "es".to_string(), "fr".to_string()];

    assert_eq!(prefs.preferred_languages.len(), 3);
}

#[test]
fn test_viewing_history_entry_creation() {
    let entry = ViewingHistoryEntry {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        content_id: Uuid::new_v4(),
        platform: Platform::Netflix,
        watched_at: Utc::now(),
        progress_seconds: 3600,
        completed: false,
        rating: Some(9),
    };

    assert_eq!(entry.platform, Platform::Netflix);
    assert_eq!(entry.progress_seconds, 3600);
    assert!(!entry.completed);
    assert_eq!(entry.rating.unwrap(), 9);
}

#[test]
fn test_viewing_history_entry_completed_with_rating() {
    let entry = ViewingHistoryEntry {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        content_id: Uuid::new_v4(),
        platform: Platform::HBOMax,
        watched_at: Utc::now(),
        progress_seconds: 7200,
        completed: true,
        rating: Some(10),
    };

    assert!(entry.completed);
    assert_eq!(entry.rating.unwrap(), 10);
}

#[test]
fn test_viewing_history_entry_no_rating() {
    let entry = ViewingHistoryEntry {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        content_id: Uuid::new_v4(),
        platform: Platform::DisneyPlus,
        watched_at: Utc::now(),
        progress_seconds: 1200,
        completed: false,
        rating: None,
    };

    assert!(entry.rating.is_none());
}
