pub mod rawg;
use reqwest::Client;
use chrono::{Utc, DateTime};
use serde::{Deserialize, Serialize};

pub struct IgdbClient {
    client: Client,
    auth_url: String,
    base_url: String,
    client_id: String,
    client_secret: String,
    token: String,
    token_expire: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize)]
pub struct AuthResponse {
    access_token: String,
    expires_in: i64,
    token_type: String,
}
