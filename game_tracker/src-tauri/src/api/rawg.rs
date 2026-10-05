use reqwest::{Client, header::{ACCEPT, AUTHORIZATION}};
use chrono::{Utc, TimeDelta};
use tauri::http::{HeaderMap, HeaderValue};
use std::{collections::HashMap, time::Duration};

use crate::{
    api::{AuthResponse, IgdbClient}, models::{SearchResult, TimeToBeatRaw, error::AppError},
};


// Need a function to build base_url and auth_url
// Need a function to call auth_url and get token and expire
// - Needs to check token_expire
// 

impl IgdbClient {
    pub fn new(client_id: String, client_secret: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to build HTTP client");

        // Build init client
        IgdbClient {
            client: client,
            auth_url: "https://id.twitch.tv/oauth2".to_string(),
            base_url: "https://api.igdb.com/v4".to_string(),
            client_id: client_id,
            client_secret: client_secret,
            token: "".to_string(),
            token_expire: None,
        }
    }

    pub async fn verify_auth(&mut self) -> Result<(), AppError>{
        
        // Check if token is valid, if exists and is still greater than current time
        // then return Ok(())
        let mut current_time = Utc::now();
        if self.token_expire.is_some() && self.token_expire.unwrap() > current_time {
            return Ok(())
        }

        // If not valid, call API to get new access token
        let auth_url = &self.auth_url;
        let client_id = &self.client_id;
        let client_secret = &self.client_secret;
        let url = format!("{auth_url}/token?client_id={client_id}&client_secret={client_secret}&grant_type=client_credentials");
        let body = self.client.post(url).send().await?;
        let json_response: AuthResponse = body.json().await?;

        // Recalculate expiration time
        let current_time = Utc::now();
        let new_expire_time = current_time + TimeDelta::seconds(json_response.expires_in);

        // Attach access token and new expiration time to Igdbclient
        let token_type = json_response.token_type;
        let access_token = json_response.access_token;
        let token = format!("{token_type} {access_token}");
        self.token = token;
        self.token_expire = Some(new_expire_time);

        Ok(())
    }

    pub async fn search_games(&self, query: &str) -> Result<Vec<SearchResult>, AppError> {
        // Validate query is non-empty
        if query.is_empty() {
            return Err(AppError::ValidationError("Query is empty".to_string()));
        }

        // Build URL
        let base_url = &self.base_url;
        let mut url = format!("{base_url}/games");

        // Build/Send request
        let headers = HeaderMap::new();
        let client_id = self.client_id;
        self.verify_auth();
        let token = self.token;
        headers.insert(ACCEPT, HeaderValue::from_str("application/json")?);
        headers.insert("Client-ID", HeaderValue::from_str(&client_id)?);
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&token)?);
        let request_body = format!("
            search \"{query}\";
            fields name,first_release_date,genres.name,platforms.name,aggregated_rating,cover.image_id;
            limit 20; "
        );
        let body = self.client.post(url).headers(headers).body(request_body).send().await?;
        let results: Vec<SearchResult> = body.json().await?;

        let ids: Vec<String> = results.iter().map(|r| r.igdb_id.to_string()).collect();
        let id_list = ids.join(",");

        url = format!("{base_url}/game_time_to_beats");
        let ttb_body = format!("fields game_id,normally; where game_id=({id_list}); limit {}", results.len());
        let ttb_response = self.client.post(url).headers(headers).body(ttb_body).send().await?;
        let ttb_results: Vec<TimeToBeatRaw> = ttb_response.json().await?;

        let playtime_by_game: HashMap<i64, i64> = ttb_results.into_iter().filter_map(|t| t.normally.map(|s| (t.game_id, s / 3600))).collect();


        for r in &mut results {
            r.avg_playtime_hours = playtime_by_game.get(&r.igdb_id).copied();
        }
        // Send request for average playtime hours
            

        // Parse JSON response
        Err(AppError::NoLaunchPath())
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
