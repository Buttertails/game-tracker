use reqwest::Client;
use std::time::Duration;

use crate::{
    api::{RawgClient, RawgResponse},
    models::{error::AppError, SearchResult},
};

impl RawgClient {
    pub fn new(api_key: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to build HTTP client");

        RawgClient {
            client: client,
            base_url: "https://api.rawg.io/api".to_string(),
            api_key: api_key.to_string(),
        }
    }

    pub fn with_base_url(api_key: String, base_url: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to build HTTP client");

        RawgClient {
            client,
            base_url,
            api_key,
        }
    }

    pub async fn search_games(&self, query: &str) -> Result<Vec<SearchResult>, AppError> {
        // Validate query is non-empty
        if query.is_empty() {
            return Err(AppError::ValidationError("Query is empty".to_string()));
        }

        // Build URL
        let base_url = &self.base_url;
        let api_key = &self.api_key;
        let url = format!("{base_url}/games?key={api_key}&search={query}&page_size=20");

        // Send GET request
        let body = self.client.get(url).send().await?;

        // Parse JSON response
        let json_response: RawgResponse = body.json().await?;

        // Map RawgGameData to SearchResult
        let search_results: Vec<SearchResult> = json_response
            .results
            .into_iter()
            .map(|game| SearchResult {
                rawg_id: game.id,
                name: game.name,
                released: game.released,
                rating: game.rating,
                platforms: game
                    .platforms
                    .unwrap_or_default()
                    .iter()
                    .map(|p| p.platform.name.clone())
                    .collect(),
                background_image: game.background_image,
            })
            .collect();

        Ok(search_results)
    }
}
