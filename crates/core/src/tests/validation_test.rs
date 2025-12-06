//! Validation tests for core validation functions

use crate::models::search::*;
use crate::types::*;
use crate::validation::*;

// SearchQuery validation tests
#[test]
fn test_search_query_validates_when_query_is_valid() {
    let query = SearchQuery::new("action movies");
    assert!(query.validate().is_ok());

    let query2 = SearchQuery::new("a");
    assert!(query2.validate().is_ok());

    let query3 = SearchQuery::new("The quick brown fox jumps over the lazy dog");
    assert!(query3.validate().is_ok());
}

#[test]
fn test_search_query_returns_error_when_query_is_empty() {
    let query = SearchQuery::new("");
    let result = query.validate();

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Query cannot be empty"));
}

#[test]
fn test_search_query_returns_error_when_query_exceeds_max_length() {
    let long_query = "a".repeat(501);
    let query = SearchQuery::new(long_query);
    let result = query.validate();

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Query too long"));
}

#[test]
fn test_search_query_validates_at_max_length_boundary() {
    let max_query = "a".repeat(500);
    let query = SearchQuery::new(max_query);
    assert!(query.validate().is_ok());

    let over_max = "a".repeat(501);
    let query2 = SearchQuery::new(over_max);
    assert!(query2.validate().is_err());
}

// Page size validation tests
#[test]
fn test_validate_page_size_success_within_bounds() {
    assert!(validate_page_size(1).is_ok());
    assert!(validate_page_size(20).is_ok());
    assert!(validate_page_size(50).is_ok());
    assert!(validate_page_size(100).is_ok());
}

#[test]
fn test_validate_page_size_error_when_zero() {
    let result = validate_page_size(0);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("must be at least 1"));
}

#[test]
fn test_validate_page_size_error_when_too_large() {
    let result = validate_page_size(101);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("too large"));

    let result2 = validate_page_size(1000);
    assert!(result2.is_err());
}

#[test]
fn test_validate_page_size_boundary_values() {
    assert!(validate_page_size(1).is_ok());
    assert!(validate_page_size(100).is_ok());
    assert!(validate_page_size(0).is_err());
    assert!(validate_page_size(101).is_err());
}

#[test]
fn test_search_query_validates_page_size() {
    let mut query = SearchQuery::new("test");
    query.page_size = 0;
    assert!(query.validate().is_err());

    query.page_size = 101;
    assert!(query.validate().is_err());

    query.page_size = 50;
    assert!(query.validate().is_ok());
}

// Rating validation tests
#[test]
fn test_validate_rating_success_within_range() {
    assert!(validate_rating(0.0).is_ok());
    assert!(validate_rating(5.0).is_ok());
    assert!(validate_rating(7.5).is_ok());
    assert!(validate_rating(10.0).is_ok());
}

#[test]
fn test_validate_rating_error_below_minimum() {
    assert!(validate_rating(-0.1).is_err());
    assert!(validate_rating(-1.0).is_err());
    assert!(validate_rating(-100.0).is_err());
}

#[test]
fn test_validate_rating_error_above_maximum() {
    assert!(validate_rating(10.1).is_err());
    assert!(validate_rating(11.0).is_err());
    assert!(validate_rating(100.0).is_err());
}

#[test]
fn test_validate_rating_boundary_values() {
    assert!(validate_rating(0.0).is_ok());
    assert!(validate_rating(10.0).is_ok());
    assert!(validate_rating(-0.001).is_err());
    assert!(validate_rating(10.001).is_err());
}

// EIDR validation tests
#[test]
fn test_validate_eidr_success_with_valid_format() {
    assert!(validate_eidr("10.5240/ABCD-1234-5678-90AB-CDEF-G").is_ok());
    assert!(validate_eidr("10.1234/XXXX-YYYY-ZZZZ-AAAA-BBBB-C").is_ok());
}

#[test]
fn test_validate_eidr_error_without_prefix() {
    let result = validate_eidr("11.5240/ABCD-1234");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("must start with '10.'"));

    assert!(validate_eidr("20.5240/ABCD").is_err());
    assert!(validate_eidr("5240/ABCD-1234").is_err());
}

#[test]
fn test_validate_eidr_error_without_slash() {
    let result = validate_eidr("10.5240-ABCD-1234");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid EIDR format"));
}

#[test]
fn test_validate_eidr_error_multiple_slashes() {
    let result = validate_eidr("10.5240/ABCD/1234");
    assert!(result.is_err());
}

// IMDb ID validation tests
#[test]
fn test_validate_imdb_id_success_with_valid_format() {
    assert!(validate_imdb_id("tt0111161").is_ok()); // 7 digits
    assert!(validate_imdb_id("tt1234567").is_ok()); // 7 digits
    assert!(validate_imdb_id("tt12345678").is_ok()); // 8 digits
    assert!(validate_imdb_id("tt123456789").is_ok()); // 9 digits
    assert!(validate_imdb_id("tt1234567890").is_ok()); // 10 digits
}

#[test]
fn test_validate_imdb_id_error_without_tt_prefix() {
    let result = validate_imdb_id("nm0000123");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("must start with 'tt'"));

    assert!(validate_imdb_id("0111161").is_err());
    assert!(validate_imdb_id("ab0111161").is_err());
}

#[test]
fn test_validate_imdb_id_error_too_few_digits() {
    assert!(validate_imdb_id("tt123").is_err());
    assert!(validate_imdb_id("tt1").is_err());
    assert!(validate_imdb_id("tt123456").is_err()); // 6 digits, need 7
}

#[test]
fn test_validate_imdb_id_error_too_many_digits() {
    assert!(validate_imdb_id("tt12345678901").is_err()); // 11 digits
    assert!(validate_imdb_id("tt123456789012345").is_err());
}

#[test]
fn test_validate_imdb_id_error_non_numeric_characters() {
    assert!(validate_imdb_id("tt0111ABC").is_err());
    assert!(validate_imdb_id("tt011116X").is_err());
    assert!(validate_imdb_id("ttABCDEFG").is_err());
}

#[test]
fn test_validate_imdb_id_boundary_values() {
    assert!(validate_imdb_id("tt0000001").is_ok()); // 7 digits (minimum)
    assert!(validate_imdb_id("tt9999999999").is_ok()); // 10 digits (maximum)
    assert!(validate_imdb_id("tt000000").is_err()); // 6 digits (too few)
    assert!(validate_imdb_id("tt99999999999").is_err()); // 11 digits (too many)
}

// SearchQuery with filters validation
#[test]
fn test_search_query_with_filters_validates() {
    let mut query = SearchQuery::new("action movies");
    query.filters.content_types = Some(vec![ContentType::Movie]);
    query.filters.genres = Some(vec![Genre::Action, Genre::SciFi]);
    query.filters.min_rating = Some(7.0);
    query.filters.max_rating = Some(10.0);

    assert!(query.validate().is_ok());
}

#[test]
fn test_search_query_with_release_year_filters() {
    let mut query = SearchQuery::new("recent movies");
    query.filters.release_year_min = Some(2020);
    query.filters.release_year_max = Some(2024);

    assert!(query.validate().is_ok());
}

#[test]
fn test_search_query_with_platform_and_region_filters() {
    let mut query = SearchQuery::new("netflix content");
    query.filters.platforms = Some(vec![Platform::Netflix, Platform::HBOMax]);
    query.filters.regions = Some(vec!["US".to_string(), "CA".to_string()]);

    assert!(query.validate().is_ok());
}

#[test]
fn test_search_query_with_maturity_rating_filter() {
    let mut query = SearchQuery::new("family movies");
    query.filters.maturity_ratings = Some(vec![
        MaturityRating::G,
        MaturityRating::PG,
        MaturityRating::PG13,
    ]);

    assert!(query.validate().is_ok());
}

// Edge case tests
#[test]
fn test_validate_query_with_special_characters() {
    assert!(validate_query("action & adventure").is_ok());
    assert!(validate_query("movies: the sequel").is_ok());
    assert!(validate_query("émile's café").is_ok());
    assert!(validate_query("日本語").is_ok());
}

#[test]
fn test_validate_query_with_whitespace() {
    assert!(validate_query("   spaces   ").is_ok());
    assert!(validate_query("\ttabs\t").is_ok());
}
