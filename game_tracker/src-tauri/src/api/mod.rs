pub mod igdb;
use reqwest::Client;
use chrono::{Utc, DateTime};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

pub struct IgdbClient {
    client: Client,
    auth_url: String,
    base_url: String,
    client_id: String,
    client_secret: String,
    token: Mutex<Token>,
}

pub struct Token {
    access_token: String,
    token_expire: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize)]
pub struct AuthResponse {
    access_token: String,
    expires_in: i64,
    token_type: String,
}
