use super::DuplicateInfo;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppError {
    ValidationError(String),
    DuplicateCategory(String),
    DuplicateTag(String),
    CategoryNotFound(i64),
    EntryNotFound(i64),
    TagNotFound(String),
    CategoryNotEmpty { category_id: i64, entry_count: i64 },
    DuplicateGameEntry(DuplicateInfo),
    ApiUnavailable(String),
    LaunchPathNotFound(String),
    LaunchFailed(String),
    NoLaunchPath,
    DatabaseError(String),
    InvalidDate(String),
}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        AppError::DatabaseError(e.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::DatabaseError(e.to_string())
    }
}

impl From<chrono::ParseError> for AppError {
    fn from(e: chrono::ParseError) -> Self {
        AppError::InvalidDate(e.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::LaunchFailed(e.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        AppError::ApiUnavailable(e.to_string())
    }
}
