pub mod error;

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub category_id: i64,
    pub name: String,
    pub is_preset: bool,
    pub display_order: i32,
    pub entry_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameEntry {
    pub entry_id: i64,
    pub name: String,
    pub category_id: i64,
    pub category_name: String,
    pub addition_date: DateTime<Utc>,
    pub tags: Vec<String>,
    pub source: Option<String>,
    pub last_played: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameEntryDetail {
    pub entry_id: i64,
    pub name: String,
    pub category_id: i64,
    pub category_name: String,
    pub addition_date: DateTime<Utc>,
    pub tags: Vec<String>,
    pub source: Option<String>,
    pub last_played: Option<NaiveDate>,
    pub launch_path: Option<String>,
    pub stored_api_data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub rawg_id: i64,
    pub name: String,
    pub released: Option<String>,
    pub rating: Option<f64>,
    pub platforms: Vec<String>,
    pub background_image: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawgGameData {
    pub id: i64,
    pub name: String,
    pub released: Option<String>,
    pub rating: Option<f64>,
    pub metacritic: Option<i32>,
    pub platforms: Option<Vec<RawgPlatform>>,
    pub genres: Option<Vec<RawgGenre>>,
    pub background_image: Option<String>,
    pub esrb_rating: Option<RawgEsrbRating>,

    #[serde(flatten)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawgPlatform {
    pub platform: RawgPlatformInner,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawgPlatformInner {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawgGenre {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawgEsrbRating {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateInfo {
    pub entry_id: i64,
    pub name: String,
    pub category_name: String,
}
