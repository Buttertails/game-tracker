pub mod error;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

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
    pub in_progress_count: i64,
    pub completed_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameEntry {
    pub entry_id: i64,
    pub shelf_id: i64,
    pub igdb_id: Option<i64>,
    pub name: String,
    pub status: GameStatus,
    pub genre: Option<String>,
    pub avg_playtime_hours: Option<i64>,
    pub release_date: Option<i32>,
    pub source: Option<String>,
    pub launch_path: Option<String>,
    pub ownership_status: OwnershipStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub addition_date: DateTime<Utc>,
    pub tags: Vec<String>,
    pub last_played: Option<DateTime<Utc>>,
    pub background_image: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameEntryDetail {
    pub entry_id: i64,
    pub shelf_id: i64,
    pub igdb_id: Option<i64>,
    pub name: String,
    pub status: GameStatus,
    pub genre: Option<String>,
    pub avg_playtime_hours: Option<i64>,
    pub release_date: Option<i32>,
    pub source: Option<String>,
    pub launch_path: Option<String>,
    pub ownership_status: OwnershipStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub addition_date: DateTime<Utc>,
    pub tags: Vec<String>,
    pub last_played: Option<DateTime<Utc>>,
    pub background_image: Option<String>,
    pub notes: Vec<TimestampedNote>,
    pub stored_api_data: Option<serde_json::Value>,
    pub api_data_version: i64,
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
    pub background_image: Option<String>,
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
    pub ownership_status: OwnershipStatus,
}

/// Neutral, API-agnostic result of a game search. This is the *input* that
/// `add_from_search` turns into a `GameEntry`. It has an external identity
/// (`igdb_id`) but no shelf, status, or user state — those only exist once the
/// game becomes a `GameEntry`.
///
/// All fields are pre-derived in the API layer so the add path is a near
/// passthrough with no RAWG/IGDB-specific parsing downstream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub igdb_id: i64,
    pub name: String,
    /// Pre-parsed from IGDB's `first_release_date` (Unix timestamp).
    pub release_date: Option<i32>,
    /// First genre name, pre-extracted for the entry's single `genre` field.
    pub genre: Option<String>,
    /// Full genre list, for display.
    pub genres: Vec<String>,
    /// Platform names, used to populate the source dropdown in the confirm form.
    pub platforms: Vec<String>,
    /// Average time-to-beat in HOURS, from IGDB's `/v4/game_time_to_beat`
    /// (`normally`, converted from seconds). `None` when IGDB has no entry.
    /// Stored as the raw value; length category is derived on demand.
    pub avg_playtime_hours: Option<i64>,
    /// IGDB rating on a 0-100 scale, for display.
    pub rating: Option<f64>,
    /// Fully-built cover URL (constructed from IGDB's `cover.image_id`).
    pub background_image: Option<String>,
    /// The minimal raw IGDB object we fetched, stored verbatim as the entry's
    /// `stored_api_data` backup (Option B). Re-fetch by `igdb_id` later for more.
    pub stored_api_data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeToBeatRaw {
    pub game_id: i64,
    pub normally: Option<i64>
}

pub fn derive_era(release_year: i32) -> Era {
    match release_year {
        y if y < 2000 => Era::Retro,
        y if y <= 2015 => Era::Modern,
        _ => Era::Recent,
    }
}

pub fn derive_length_category(playtime: i64) -> LengthCategory {
    match playtime {
        p if p < 10 => LengthCategory::Short,
        p if p <= 30 => LengthCategory::Medium,
        _ => LengthCategory::Long,
    }
}

pub fn format_duration(started_at: DateTime<Utc>, completed_at: DateTime<Utc>) -> String {
    let duration = completed_at.signed_duration_since(started_at);
    let total_hours = duration.num_hours();

    if total_hours < 1 {
        "less than 1 hour".to_string()
    } else {
        let days = total_hours / 24;
        let hours = total_hours % 24;
        if days > 0 {
            format!("{}d {}h", days, hours)
        } else {
            format!("{}h", hours)
        }
    }
}

impl GameStatus {
    pub fn from_id(id: i64) -> Option<Self> {
        match id {
            1 => Some(GameStatus::Backlog),
            2 => Some(GameStatus::InProgress),
            3 => Some(GameStatus::Completed),
            _ => None,
        }
    }

    pub fn to_id(&self) -> i64 {
        match self {
            GameStatus::Backlog => 1,
            GameStatus::InProgress => 2,
            GameStatus::Completed => 3,
        }
    }
}

impl LengthCategory {
    pub fn from_id(id: i64) -> Option<Self> {
        match id {
            1 => Some(LengthCategory::Short),
            2 => Some(LengthCategory::Medium),
            3 => Some(LengthCategory::Long),
            _ => None,
        }
    }

    pub fn to_id(&self) -> i64 {
        match self {
            LengthCategory::Short => 1,
            LengthCategory::Medium => 2,
            LengthCategory::Long => 3,
        }
    }
}

impl Era {
    pub fn from_id(id: i64) -> Option<Self> {
        match id {
            1 => Some(Era::Retro),
            2 => Some(Era::Modern),
            3 => Some(Era::Recent),
            _ => None,
        }
    }

    pub fn to_id(&self) -> i64 {
        match self {
            Era::Retro => 1,
            Era::Modern => 2,
            Era::Recent => 3,
        }
    }
}

impl OwnershipStatus {
    pub fn from_id(id: i64) -> Option<Self> {
        match id {
            1 => Some(OwnershipStatus::Installed),
            2 => Some(OwnershipStatus::NotInstalled),
            3 => Some(OwnershipStatus::Wishlisted),
            _ => None,
        }
    }

    pub fn to_id(&self) -> i64 {
        match self {
            OwnershipStatus::Installed => 1,
            OwnershipStatus::NotInstalled => 2,
            OwnershipStatus::Wishlisted => 3,
        }
    }
}

impl fmt::Display for GameStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GameStatus::Backlog => write!(f, "Backlog"),
            GameStatus::InProgress => write!(f, "In Progress"),
            GameStatus::Completed => write!(f, "Completed"),
        }
    }
}
