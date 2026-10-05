use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppError {
    ValidationError(String),
    DuplicateShelf(String),
    DuplicateTag(String),
    ShelfNotFound(i64),
    EntryNotFound(i64),
    NoteNotFound(String),
    TagNotFound(String),
    InProgressFull {
        shelf_id: i64,
        cap: u32,
    },
    InvalidTransition {
        from: String,
        to: String,
        reason: String,
    },
    ConfirmationRequired(String),
    ApiUnavailable(String),
    LaunchPathNotFound(String),
    LaunchFailed(String),
    NoLaunchPath,
    DatabaseError(String),
    SmartFillNotEligible(String),
    SmartFillNoSlots(String),
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

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<tauri::http::header::InvalidHeaderValue> for AppError {
    fn from(e: tauri::http::header::InvalidHeaderValue) -> Self {
        AppError::ApiUnavailable(e.to_string())
    }
}

impl From<std::sync::PoisonError<std::sync::MutexGuard<'_, crate::api::Token>>> for AppError {
    fn from(e: std::sync::PoisonError<std::sync::MutexGuard<'_, crate::api::Token>>) -> Self {
        AppError::ApiUnavailable(e.to_string())
    }
}   
