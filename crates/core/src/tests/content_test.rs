//! Content model tests

use crate::models::content::*;
use crate::types::*;
use chrono::{Duration, Utc};
use uuid::Uuid;

#[test]
fn test_canonical_content_creation_with_full_metadata() {
    let content_id = Uuid::new_v4();
    let now = Utc::now();

    let content = CanonicalContent {
        id: content_id,
        title: "The Shawshank Redemption".to_string(),
        content_type: ContentType::Movie,
        release_date: Some(now),
        genres: vec![Genre::Drama, Genre::Crime],
        maturity_rating: Some(MaturityRating::R),
        external_ids: ExternalIds {
            imdb_id: Some("tt0111161".to_string()),
            tmdb_id: Some(278),
            eidr: Some("10.5240/ABCD-1234-5678-90AB-CDEF-G".to_string()),
            tvdb_id: None,
        },
        availability: vec![],
        metadata: ContentMetadata {
            description: Some("Two imprisoned men bond over a number of years".to_string()),
            duration_minutes: Some(142),
            director: Some(vec!["Frank Darabont".to_string()]),
            cast: Some(vec!["Tim Robbins".to_string(), "Morgan Freeman".to_string()]),
            production_companies: Some(vec!["Castle Rock Entertainment".to_string()]),
            original_language: Some("en".to_string()),
            series_metadata: None,
        },
        created_at: now,
        updated_at: now,
    };

    assert_eq!(content.id, content_id);
    assert_eq!(content.title, "The Shawshank Redemption");
    assert_eq!(content.content_type, ContentType::Movie);
    assert_eq!(content.genres.len(), 2);
    assert_eq!(content.external_ids.imdb_id.as_ref().unwrap(), "tt0111161");
    assert_eq!(content.metadata.duration_minutes.unwrap(), 142);
}

#[test]
fn test_external_ids_validation_imdb_format() {
    let ids = ExternalIds {
        imdb_id: Some("tt0111161".to_string()),
        tmdb_id: Some(278),
        eidr: None,
        tvdb_id: None,
    };

    let imdb = ids.imdb_id.unwrap();
    assert!(imdb.starts_with("tt"));
    assert_eq!(imdb.len(), 9); // "tt" + 7 digits
}

#[test]
fn test_external_ids_validation_eidr_format() {
    let ids = ExternalIds {
        imdb_id: None,
        tmdb_id: None,
        eidr: Some("10.5240/ABCD-1234-5678-90AB-CDEF-G".to_string()),
        tvdb_id: None,
    };

    let eidr = ids.eidr.unwrap();
    assert!(eidr.starts_with("10."));
    assert!(eidr.contains('/'));
}

#[test]
fn test_platform_availability_is_currently_available_no_restrictions() {
    let availability = PlatformAvailability {
        platform: Platform::Netflix,
        availability_type: AvailabilityType::Subscription,
        region: "US".to_string(),
        url: Some("https://netflix.com/title/123".to_string()),
        price: None,
        currency: None,
        available_from: None,
        available_until: None,
        video_quality: vec![VideoQuality::HD, VideoQuality::UHD],
        audio_quality: vec![AudioQuality::Surround51, AudioQuality::Atmos],
    };

    assert!(availability.is_currently_available());
}

#[test]
fn test_platform_availability_is_currently_available_within_window() {
    let now = Utc::now();
    let yesterday = now - Duration::days(1);
    let tomorrow = now + Duration::days(1);

    let availability = PlatformAvailability {
        platform: Platform::HBOMax,
        availability_type: AvailabilityType::Subscription,
        region: "US".to_string(),
        url: None,
        price: None,
        currency: None,
        available_from: Some(yesterday),
        available_until: Some(tomorrow),
        video_quality: vec![VideoQuality::UHD],
        audio_quality: vec![AudioQuality::Atmos],
    };

    assert!(availability.is_currently_available());
}

#[test]
fn test_platform_availability_not_available_before_start_date() {
    let future = Utc::now() + Duration::days(10);

    let availability = PlatformAvailability {
        platform: Platform::DisneyPlus,
        availability_type: AvailabilityType::Subscription,
        region: "CA".to_string(),
        url: None,
        price: None,
        currency: None,
        available_from: Some(future),
        available_until: None,
        video_quality: vec![VideoQuality::UHD],
        audio_quality: vec![AudioQuality::Atmos],
    };

    assert!(!availability.is_currently_available());
}

#[test]
fn test_platform_availability_not_available_after_end_date() {
    let past = Utc::now() - Duration::days(10);

    let availability = PlatformAvailability {
        platform: Platform::Hulu,
        availability_type: AvailabilityType::Subscription,
        region: "US".to_string(),
        url: None,
        price: None,
        currency: None,
        available_from: None,
        available_until: Some(past),
        video_quality: vec![VideoQuality::HD],
        audio_quality: vec![AudioQuality::Stereo],
    };

    assert!(!availability.is_currently_available());
}

#[test]
fn test_platform_availability_rental_with_price() {
    let availability = PlatformAvailability {
        platform: Platform::PrimeVideo,
        availability_type: AvailabilityType::Rental,
        region: "US".to_string(),
        url: Some("https://primevideo.com/rent/123".to_string()),
        price: Some(3.99),
        currency: Some("USD".to_string()),
        available_from: None,
        available_until: None,
        video_quality: vec![VideoQuality::HD, VideoQuality::UHD],
        audio_quality: vec![AudioQuality::Surround51],
    };

    assert_eq!(availability.availability_type, AvailabilityType::Rental);
    assert_eq!(availability.price.unwrap(), 3.99);
    assert_eq!(availability.currency.as_ref().unwrap(), "USD");
}

#[test]
fn test_platform_availability_multiple_quality_options() {
    let availability = PlatformAvailability {
        platform: Platform::AppleTVPlus,
        availability_type: AvailabilityType::Purchase,
        region: "GB".to_string(),
        url: None,
        price: Some(12.99),
        currency: Some("GBP".to_string()),
        available_from: None,
        available_until: None,
        video_quality: vec![VideoQuality::SD, VideoQuality::HD, VideoQuality::UHD, VideoQuality::HDR],
        audio_quality: vec![AudioQuality::Stereo, AudioQuality::Surround51, AudioQuality::Atmos],
    };

    assert_eq!(availability.video_quality.len(), 4);
    assert_eq!(availability.audio_quality.len(), 3);
    assert!(availability.video_quality.contains(&VideoQuality::HDR));
    assert!(availability.audio_quality.contains(&AudioQuality::Atmos));
}

#[test]
fn test_series_metadata_complete() {
    let series_id = Uuid::new_v4();
    let metadata = SeriesMetadata {
        season_number: Some(3),
        episode_number: Some(12),
        total_seasons: Some(5),
        total_episodes: Some(60),
        series_id: Some(series_id),
    };

    assert_eq!(metadata.season_number.unwrap(), 3);
    assert_eq!(metadata.episode_number.unwrap(), 12);
    assert_eq!(metadata.total_seasons.unwrap(), 5);
    assert_eq!(metadata.total_episodes.unwrap(), 60);
    assert_eq!(metadata.series_id.unwrap(), series_id);
}

#[test]
fn test_series_metadata_partial() {
    let metadata = SeriesMetadata {
        season_number: Some(1),
        episode_number: Some(1),
        total_seasons: None,
        total_episodes: None,
        series_id: None,
    };

    assert!(metadata.total_seasons.is_none());
    assert!(metadata.total_episodes.is_none());
}

#[test]
fn test_content_metadata_minimal() {
    let metadata = ContentMetadata {
        description: None,
        duration_minutes: Some(90),
        director: None,
        cast: None,
        production_companies: None,
        original_language: Some("en".to_string()),
        series_metadata: None,
    };

    assert_eq!(metadata.duration_minutes.unwrap(), 90);
    assert!(metadata.description.is_none());
    assert!(metadata.cast.is_none());
}

#[test]
fn test_content_metadata_with_large_cast() {
    let cast = vec![
        "Actor 1".to_string(),
        "Actor 2".to_string(),
        "Actor 3".to_string(),
        "Actor 4".to_string(),
        "Actor 5".to_string(),
    ];

    let metadata = ContentMetadata {
        description: Some("A film with a large ensemble cast".to_string()),
        duration_minutes: Some(180),
        director: Some(vec!["Director A".to_string(), "Director B".to_string()]),
        cast: Some(cast.clone()),
        production_companies: Some(vec!["Studio X".to_string()]),
        original_language: Some("en".to_string()),
        series_metadata: None,
    };

    assert_eq!(metadata.cast.unwrap().len(), 5);
    assert_eq!(metadata.director.unwrap().len(), 2);
}

#[test]
fn test_canonical_content_serialization() {
    let content = CanonicalContent {
        id: Uuid::new_v4(),
        title: "Test".to_string(),
        content_type: ContentType::Movie,
        release_date: None,
        genres: vec![Genre::Action],
        maturity_rating: None,
        external_ids: ExternalIds::default(),
        availability: vec![],
        metadata: ContentMetadata {
            description: None,
            duration_minutes: Some(90),
            director: None,
            cast: None,
            production_companies: None,
            original_language: None,
            series_metadata: None,
        },
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let json = serde_json::to_string(&content).unwrap();
    let deserialized: CanonicalContent = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.title, content.title);
    assert_eq!(deserialized.content_type, content.content_type);
}
