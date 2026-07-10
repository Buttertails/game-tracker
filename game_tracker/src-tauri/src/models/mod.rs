pub mod error;

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameStatus {
    Backlog,
    InProgress,
    Completed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LengthCategory {
    Short,  // < 10 hours
    Medium, // 10-30 hours
    Long,   // > 30 hours
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Era {
    Retro,  // before 2000,
    Modern, // 2000-2015,
    Recent, // after 2015
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OwnershipStatus {
    Installed,
    NotInstalled,
    Wishlisted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shelf {
    pub shelf_id: i64,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShelfSummary {
    pub shelf_id: i64,
    pub name: String,
    pub backlog_count: i64,
    pub in_progres_count: i64,
    pub completed_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameEntry {
    pub entry_id: i64,
    pub shelf_id: i64,
    pub name: String,
    pub status: GameStatus,
    pub genre: Option<String>,
    pub length_category: Option<LengthCategory>,
    pub release_year: Option<Era>,
    pub source: Option<String>,
    pub launch_path: Option<String>,
    pub ownership_status: Option<OwnershipStatus>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub addition_date: DateTime<Utc>,
    pub tags: Vec<String>,
    pub last_played: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameEntryDetail {
    pub entry_id: i64,
    pub shelf_id: i64,
    pub name: String,
    pub status: GameStatus,
    pub genre: Option<String>,
    pub length_category: Option<LengthCategory>,
    pub release_year: Option<Era>,
    pub source: Option<String>,
    pub launch_path: Option<String>,
    pub ownership_status: Option<OwnershipStatus>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub addition_date: DateTime<Utc>,
    pub tags: Vec<String>,
    pub last_played: Option<NaiveDate>,
    pub notes: Vec<TimestampedNote>,
    pub stored_api_data: Option<serde_json::Value>,
    pub completion_duration: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShelfEntries {
    pub shelf: Shelf,
    pub backlog: Vec<GameEntry>,
    pub in_progress: Vec<GameEntry>,
    pub completed: Vec<GameEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimestampedNote {
    pub note_id: i64,
    pub entry_id: i64,
    pub text: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartFillSuggestion {
    pub entry_id: i64,
    pub name: String,
    pub genre: String,
    pub length_category: LengthCategory,
    pub era: Era,
    pub diversity_score: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualEntryInput {
    pub name: String,
    pub shelf_id: i64,
    pub genre: Option<String>,
    pub length_category: Option<LengthCategory>,
    pub release_year: Option<i32>,
    pub source: Option<String>,
    pub launch_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub rawg_id: i64,
    pub name: String,
    pub released: Option<String>,
    pub rating: Option<f64>,
    pub platforms: Vec<String>,
    pub genres: Vec<String>,
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

pub fn derive_era(release_year: i32) -> Era {
    match release_year {
        y if y < 2000 => Era::Retro,
        y if y <= 2015 => Era::Modern,
        _ => Era::Recent,
    }
}

pub fn format_duration(started_at: DateTime<Utc>, completed_at: DateTime<Utc>) -> String {
    let duration = completed_at.signed_duration_since(started_at);
    let total_horus = duration.num_hours();

    if total_horus < 1 {
        "less than 1 hour".to_string()
    } else {
        let days = total_horus / 24;
        let hours = total_horus % 24;
        if days > 0 {
            format!("{}d {}h", days, hours)
        } else {
            format!("{}h", hours)
        }
    }
}
