pub mod rawg;
use reqwest::Client;

use crate::models::RawgGameData;
use serde::Deserialize;

pub struct RawgClient {
    client: Client,
    base_url: String,
    api_key: String,
}

#[derive(Deserialize)]
pub struct RawgResponse {
    results: Vec<RawgGameData>,
}
