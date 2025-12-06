use serde::{Deserialize, Serialize};

/// Search filters
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchFilters {
    /// Genre filters (OR logic)
    pub genres: Vec<String>,

    /// Platform availability filters
    pub platforms: Vec<String>,

    /// Year range filter (min, max)
    pub year_range: Option<(i32, i32)>,

    /// Rating range filter (min, max)
    pub rating_range: Option<(f32, f32)>,
}

impl SearchFilters {
    /// Check if any filters are active
    pub fn is_empty(&self) -> bool {
        self.genres.is_empty()
            && self.platforms.is_empty()
            && self.year_range.is_none()
            && self.rating_range.is_none()
    }

    /// Build SQL WHERE clause for filters
    pub fn to_sql_where_clause(&self) -> (String, Vec<String>) {
        let mut conditions = Vec::new();
        let mut params: Vec<String> = Vec::new();

        // Genre filter
        if !self.genres.is_empty() {
            conditions.push(format!("genres && ARRAY[{}]::text[]",
                self.genres.iter()
                    .map(|g| format!("'{}'", g))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        // Platform filter
        if !self.platforms.is_empty() {
            conditions.push(format!("platforms && ARRAY[{}]::text[]",
                self.platforms.iter()
                    .map(|p| format!("'{}'", p))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        // Year range filter
        if let Some((min_year, max_year)) = self.year_range {
            conditions.push(format!(
                "release_year BETWEEN {} AND {}",
                min_year, max_year
            ));
        }

        // Rating range filter
        if let Some((min_rating, max_rating)) = self.rating_range {
            conditions.push(format!(
                "average_rating BETWEEN {} AND {}",
                min_rating, max_rating
            ));
        }

        let clause = if conditions.is_empty() {
            "1=1".to_string()
        } else {
            conditions.join(" AND ")
        };

        (clause, params)
    }

    /// Estimate filter selectivity (0.0 = very selective, 1.0 = not selective)
    pub fn estimate_selectivity(&self) -> f32 {
        let mut selectivity = 1.0;

        // Genre filter reduces to ~30% of content
        if !self.genres.is_empty() {
            selectivity *= 0.3;
        }

        // Platform filter reduces to ~40% of content
        if !self.platforms.is_empty() {
            selectivity *= 0.4;
        }

        // Year range filter
        if let Some((min_year, max_year)) = self.year_range {
            let range = (max_year - min_year) as f32;
            selectivity *= (range / 100.0).min(1.0); // Assume 100 year catalog
        }

        // Rating filter
        if self.rating_range.is_some() {
            selectivity *= 0.5;
        }

        selectivity
    }

    /// Determine if pre-filtering or post-filtering is better
    pub fn should_pre_filter(&self) -> bool {
        self.estimate_selectivity() < 0.1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_filters() {
        let filters = SearchFilters::default();
        assert!(filters.is_empty());
    }

    #[test]
    fn test_sql_where_clause() {
        let filters = SearchFilters {
            genres: vec!["action".to_string(), "thriller".to_string()],
            platforms: vec!["netflix".to_string()],
            year_range: Some((2020, 2024)),
            rating_range: Some((7.0, 10.0)),
        };

        let (clause, _) = filters.to_sql_where_clause();
        assert!(clause.contains("genres &&"));
        assert!(clause.contains("platforms &&"));
        assert!(clause.contains("BETWEEN"));
    }

    #[test]
    fn test_selectivity_estimation() {
        let filters = SearchFilters {
            genres: vec!["action".to_string()],
            platforms: vec!["netflix".to_string()],
            year_range: Some((2020, 2024)),
            rating_range: None,
        };

        let selectivity = filters.estimate_selectivity();
        assert!(selectivity < 0.1); // Should be highly selective
        assert!(filters.should_pre_filter());
    }
}
