use super::DuplicateInfo;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppError {
    ValidationError(String),
    DuplicateCategory(String),
    DuplicateTag(String),
    CategoryNotFound(i64),
    EntryNotFound(i64),
    CategoryNotEmpty { category_id: i64, entry_count: i64 },
    DuplicateGameEntry(DuplicateInfo),
    ApiUnavailable(String),
    LaunchPathNotFound(String),
    LaunchFailed(String),
    NoLaunchPath,
    DatabaseError(String),
    InvalidDate(String),
}
