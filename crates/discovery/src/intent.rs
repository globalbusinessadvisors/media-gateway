use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Natural Language Intent Parser
/// Extracts search intent from user queries
pub struct IntentParser {
    /// GPT-4o-mini API client
    client: reqwest::Client,

    /// API configuration
    api_url: String,
    api_key: String,
}

/// Parsed search intent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedIntent {
    /// Mood/emotional tone keywords
    pub mood: Vec<String>,

    /// Theme keywords
    pub themes: Vec<String>,

    /// Referenced titles (for "like X" queries)
    pub references: Vec<String>,

    /// Extracted filters
    pub filters: IntentFilters,

    /// Fallback query string
    pub fallback_query: String,

    /// Confidence score (0.0-1.0)
    pub confidence: f32,
}

/// Intent filters extracted from query
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IntentFilters {
    pub genre: Vec<String>,
    pub platform: Vec<String>,
    pub year_range: Option<(i32, i32)>,
}

/// Intent type classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntentType {
    /// Direct search query
    Search,

    /// Recommendation request
    Recommendation,

    /// Trivia/information query
    Trivia,
}

impl IntentParser {
    /// Create new intent parser
    pub fn new(api_url: String, api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_url,
            api_key,
        }
    }

    /// Parse natural language query into structured intent
    pub async fn parse(&self, query: &str) -> anyhow::Result<ParsedIntent> {
        // Check cache first (TODO: implement caching)

        // Try GPT parsing
        match self.parse_with_gpt(query).await {
            Ok(intent) => Ok(intent),
            Err(e) => {
                tracing::warn!("GPT parsing failed, using fallback: {}", e);
                Ok(self.fallback_parse(query))
            }
        }
    }

    /// Parse using GPT-4o-mini
    async fn parse_with_gpt(&self, query: &str) -> anyhow::Result<ParsedIntent> {
        let prompt = self.build_prompt(query);

        let request = serde_json::json!({
            "model": "gpt-4o-mini",
            "messages": [
                {
                    "role": "system",
                    "content": INTENT_PARSER_SYSTEM_PROMPT
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": 0.3,
            "response_format": { "type": "json_object" }
        });

        let response = self
            .client
            .post(&self.api_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?;

        let response_body: serde_json::Value = response.json().await?;

        // Extract content from GPT response
        let content = response_body["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid GPT response"))?;

        let intent: ParsedIntent = serde_json::from_str(content)?;

        // Validate
        if intent.confidence < 0.0 || intent.confidence > 1.0 {
            anyhow::bail!("Invalid confidence score");
        }

        Ok(intent)
    }

    /// Fallback parsing using simple pattern matching
    fn fallback_parse(&self, query: &str) -> ParsedIntent {
        let query_lower = query.to_lowercase();
        let tokens: Vec<&str> = query_lower.split_whitespace().collect();

        // Extract genres
        let genres = self.extract_genres(&tokens);

        // Extract platforms
        let platforms = self.extract_platforms(&tokens);

        // Extract references (simple "like X" pattern)
        let references = self.extract_references(query);

        ParsedIntent {
            mood: Vec::new(),
            themes: Vec::new(),
            references,
            filters: IntentFilters {
                genre: genres,
                platform: platforms,
                year_range: None,
            },
            fallback_query: query.to_string(),
            confidence: 0.5,
        }
    }

    /// Build GPT prompt
    fn build_prompt(&self, query: &str) -> String {
        format!(
            r#"Analyze this media search query and extract structured information:

Query: "{}"

Extract:
1. Mood/Vibes: emotional tone (e.g., "dark", "uplifting", "tense")
2. Themes: main subjects (e.g., "heist", "romance", "sci-fi")
3. References: "similar to X" or "like Y" mentions
4. Filters: platform, genre, year constraints
5. Confidence: 0.0-1.0 score for extraction quality

Return JSON:
{{
  "mood": ["mood1", "mood2"],
  "themes": ["theme1", "theme2"],
  "references": ["title1", "title2"],
  "filters": {{
    "genre": ["genre1"],
    "platform": ["platform1"],
    "year_range": {{"min": 2020, "max": 2024}}
  }},
  "fallback_query": "simplified query string",
  "confidence": 0.85
}}"#,
            query
        )
    }

    /// Extract genres from tokens
    fn extract_genres(&self, tokens: &[&str]) -> Vec<String> {
        let genre_keywords: HashMap<&str, &str> = [
            ("action", "action"),
            ("comedy", "comedy"),
            ("drama", "drama"),
            ("horror", "horror"),
            ("thriller", "thriller"),
            ("romance", "romance"),
            ("sci-fi", "science_fiction"),
            ("scifi", "science_fiction"),
            ("fantasy", "fantasy"),
            ("documentary", "documentary"),
        ]
        .iter()
        .cloned()
        .collect();

        tokens
            .iter()
            .filter_map(|&token| genre_keywords.get(token).map(|&g| g.to_string()))
            .collect()
    }

    /// Extract platforms from tokens
    fn extract_platforms(&self, tokens: &[&str]) -> Vec<String> {
        let platform_keywords: HashMap<&str, &str> = [
            ("netflix", "netflix"),
            ("prime", "prime_video"),
            ("hulu", "hulu"),
            ("disney", "disney_plus"),
            ("hbo", "hbo_max"),
        ]
        .iter()
        .cloned()
        .collect();

        tokens
            .iter()
            .filter_map(|&token| platform_keywords.get(token).map(|&p| p.to_string()))
            .collect()
    }

    /// Extract title references
    fn extract_references(&self, query: &str) -> Vec<String> {
        let mut references = Vec::new();

        // Pattern: "like The Matrix"
        if let Some(caps) = regex::Regex::new(r"like\s+([A-Z][a-zA-Z0-9\s]+)")
            .ok()
            .and_then(|re| re.captures(query))
        {
            if let Some(title) = caps.get(1) {
                references.push(title.as_str().trim().to_string());
            }
        }

        // Pattern: "similar to Inception"
        if let Some(caps) = regex::Regex::new(r"similar to\s+([A-Z][a-zA-Z0-9\s]+)")
            .ok()
            .and_then(|re| re.captures(query))
        {
            if let Some(title) = caps.get(1) {
                references.push(title.as_str().trim().to_string());
            }
        }

        references
    }
}

/// System prompt for GPT intent parsing
const INTENT_PARSER_SYSTEM_PROMPT: &str = r#"You are a media search intent parser.
Extract structured information from user queries about movies and TV shows.
Focus on mood, themes, references to other content, and filters.
Return valid JSON matching the specified schema.
Be conservative with confidence scores - only give high scores when intent is very clear."#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fallback_parse_genres() {
        let parser = IntentParser::new(String::new(), String::new());
        let intent = parser.fallback_parse("action comedy movies");

        assert_eq!(intent.filters.genre, vec!["action", "comedy"]);
    }

    #[test]
    fn test_fallback_parse_platforms() {
        let parser = IntentParser::new(String::new(), String::new());
        let intent = parser.fallback_parse("netflix shows");

        assert_eq!(intent.filters.platform, vec!["netflix"]);
    }

    #[test]
    fn test_extract_references() {
        let parser = IntentParser::new(String::new(), String::new());
        let references = parser.extract_references("movies like The Matrix");

        assert_eq!(references, vec!["The Matrix"]);
    }
}
