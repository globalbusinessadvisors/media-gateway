//! Type tests for core enums and type definitions

use crate::types::*;
use serde_json;

#[test]
fn test_content_type_serialization_all_variants() {
    let test_cases = vec![
        (ContentType::Movie, r#""movie""#),
        (ContentType::Series, r#""series""#),
        (ContentType::Episode, r#""episode""#),
        (ContentType::Short, r#""short""#),
        (ContentType::Documentary, r#""documentary""#),
        (ContentType::Special, r#""special""#),
    ];

    for (content_type, expected_json) in test_cases {
        let json = serde_json::to_string(&content_type).unwrap();
        assert_eq!(json, expected_json);

        let deserialized: ContentType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, content_type);
    }
}

#[test]
fn test_platform_enum_coverage_all_11_variants() {
    let platforms = vec![
        Platform::Netflix,
        Platform::PrimeVideo,
        Platform::DisneyPlus,
        Platform::Hulu,
        Platform::AppleTVPlus,
        Platform::HBOMax,
        Platform::Peacock,
        Platform::ParamountPlus,
        Platform::YouTube,
        Platform::Crave,
        Platform::BBCiPlayer,
    ];

    assert_eq!(platforms.len(), 11, "Must test all 11 platform variants");

    for platform in platforms {
        let json = serde_json::to_string(&platform).unwrap();
        let deserialized: Platform = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, platform);
    }
}

#[test]
fn test_platform_serialization_snake_case() {
    let test_cases = vec![
        (Platform::Netflix, r#""netflix""#),
        (Platform::PrimeVideo, r#""prime_video""#),
        (Platform::DisneyPlus, r#""disney_plus""#),
        (Platform::AppleTVPlus, r#""apple_tv_plus""#),
        (Platform::HBOMax, r#""hbo_max""#),
        (Platform::ParamountPlus, r#""paramount_plus""#),
        (Platform::BBCiPlayer, r#""bbc_i_player""#),
    ];

    for (platform, expected_json) in test_cases {
        let json = serde_json::to_string(&platform).unwrap();
        assert_eq!(json, expected_json);
    }
}

#[test]
fn test_genre_mapping_completeness() {
    let genres = vec![
        Genre::Action,
        Genre::Adventure,
        Genre::Animation,
        Genre::Comedy,
        Genre::Crime,
        Genre::Documentary,
        Genre::Drama,
        Genre::Family,
        Genre::Fantasy,
        Genre::Horror,
        Genre::Mystery,
        Genre::Romance,
        Genre::SciFi,
        Genre::Thriller,
        Genre::Western,
        Genre::Musical,
        Genre::War,
        Genre::Biography,
        Genre::History,
        Genre::Sport,
        Genre::GameShow,
        Genre::RealityTV,
        Genre::TalkShow,
        Genre::News,
    ];

    assert_eq!(genres.len(), 24, "Must have all 24 genre variants");

    for genre in genres {
        let json = serde_json::to_string(&genre).unwrap();
        let deserialized: Genre = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, genre);
    }
}

#[test]
fn test_genre_serialization_snake_case() {
    let test_cases = vec![
        (Genre::Action, r#""action""#),
        (Genre::SciFi, r#""sci_fi""#),
        (Genre::GameShow, r#""game_show""#),
        (Genre::RealityTV, r#""reality_tv""#),
        (Genre::TalkShow, r#""talk_show""#),
    ];

    for (genre, expected_json) in test_cases {
        let json = serde_json::to_string(&genre).unwrap();
        assert_eq!(json, expected_json);
    }
}

#[test]
fn test_availability_type_all_variants() {
    let types = vec![
        AvailabilityType::Subscription,
        AvailabilityType::Rental,
        AvailabilityType::Purchase,
        AvailabilityType::Free,
    ];

    for availability_type in types {
        let json = serde_json::to_string(&availability_type).unwrap();
        let deserialized: AvailabilityType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, availability_type);
    }
}

#[test]
fn test_video_quality_ordering() {
    assert!(VideoQuality::HD > VideoQuality::SD);
    assert!(VideoQuality::UHD > VideoQuality::HD);
    assert!(VideoQuality::HDR > VideoQuality::UHD);
}

#[test]
fn test_video_quality_serialization_uppercase() {
    let test_cases = vec![
        (VideoQuality::SD, r#""SD""#),
        (VideoQuality::HD, r#""HD""#),
        (VideoQuality::UHD, r#""UHD""#),
        (VideoQuality::HDR, r#""HDR""#),
    ];

    for (quality, expected_json) in test_cases {
        let json = serde_json::to_string(&quality).unwrap();
        assert_eq!(json, expected_json);
    }
}

#[test]
fn test_audio_quality_ordering() {
    assert!(AudioQuality::Surround51 > AudioQuality::Stereo);
    assert!(AudioQuality::Surround71 > AudioQuality::Surround51);
    assert!(AudioQuality::Atmos > AudioQuality::Surround71);
    assert!(AudioQuality::DtsX > AudioQuality::Surround71);
}

#[test]
fn test_audio_quality_all_variants() {
    let qualities = vec![
        AudioQuality::Stereo,
        AudioQuality::Surround51,
        AudioQuality::Surround71,
        AudioQuality::Atmos,
        AudioQuality::DtsX,
    ];

    for quality in qualities {
        let json = serde_json::to_string(&quality).unwrap();
        let deserialized: AudioQuality = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, quality);
    }
}

#[test]
fn test_subtitle_format_all_variants() {
    let formats = vec![
        SubtitleFormat::ClosedCaptions,
        SubtitleFormat::SDH,
        SubtitleFormat::Standard,
    ];

    for format in formats {
        let json = serde_json::to_string(&format).unwrap();
        let deserialized: SubtitleFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, format);
    }
}

#[test]
fn test_maturity_rating_ordering() {
    assert!(MaturityRating::PG > MaturityRating::G);
    assert!(MaturityRating::PG13 > MaturityRating::PG);
    assert!(MaturityRating::R > MaturityRating::PG13);
    assert!(MaturityRating::NC17 > MaturityRating::R);
    assert!(MaturityRating::TVMA > MaturityRating::TV14);
}

#[test]
fn test_maturity_rating_serialization_uppercase() {
    let test_cases = vec![
        (MaturityRating::G, r#""G""#),
        (MaturityRating::PG, r#""PG""#),
        (MaturityRating::PG13, r#""PG13""#),
        (MaturityRating::TVMA, r#""TVMA""#),
    ];

    for (rating, expected_json) in test_cases {
        let json = serde_json::to_string(&rating).unwrap();
        assert_eq!(json, expected_json);
    }
}

#[test]
fn test_maturity_rating_all_variants() {
    let ratings = vec![
        MaturityRating::G,
        MaturityRating::PG,
        MaturityRating::PG13,
        MaturityRating::R,
        MaturityRating::NC17,
        MaturityRating::NR,
        MaturityRating::TVY,
        MaturityRating::TVY7,
        MaturityRating::TVG,
        MaturityRating::TVPG,
        MaturityRating::TV14,
        MaturityRating::TVMA,
    ];

    assert_eq!(ratings.len(), 12, "Must have all maturity rating variants");

    for rating in ratings {
        let json = serde_json::to_string(&rating).unwrap();
        let deserialized: MaturityRating = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, rating);
    }
}

#[test]
fn test_region_type_alias() {
    let region: Region = "US".to_string();
    assert_eq!(region.len(), 2);

    let region_ca: Region = "CA".to_string();
    assert_eq!(region_ca, "CA");
}

#[test]
fn test_enum_equality_and_hash() {
    use std::collections::HashSet;

    let mut platforms = HashSet::new();
    platforms.insert(Platform::Netflix);
    platforms.insert(Platform::Netflix); // Duplicate

    assert_eq!(platforms.len(), 1);
    assert!(platforms.contains(&Platform::Netflix));

    let mut genres = HashSet::new();
    genres.insert(Genre::Action);
    genres.insert(Genre::Drama);
    genres.insert(Genre::Action); // Duplicate

    assert_eq!(genres.len(), 2);
}

#[test]
fn test_enum_clone_and_copy() {
    let content_type = ContentType::Movie;
    let cloned = content_type;
    assert_eq!(content_type, cloned);

    let platform = Platform::Netflix;
    let copied = platform;
    assert_eq!(platform, copied);
}
