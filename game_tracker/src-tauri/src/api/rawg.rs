use reqwest::Client;
use std::time::Duration;

use crate::{
    api::{RawgClient, RawgResponse},
    models::{error::AppError, RawgGameData},
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

    pub async fn search_games(&self, query: &str) -> Result<Vec<RawgGameData>, AppError> {
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

        Ok(json_response.results)
    }
}
