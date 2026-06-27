use crate::{
    db::{game_entries, Database},
    models::error::AppError,
};
use chrono::{Local, NaiveDate};
pub struct SourceService<'a> {
    db: &'a Database,
}

impl<'a> SourceService<'a> {
    pub fn new(db: &'a Database) -> Self {
        SourceService { db }
    }

    pub fn update_source(&self, entry_id: i64, source: Option<&str>) -> Result<(), AppError> {
        let value = match source {
            Some(s) if !s.trim().is_empty() => {
                let trimmed = s.trim();
                if trimmed.len() > 100 {
                    return Err(AppError::ValidationError(
                        "Source must be 100 characters or less".to_string(),
                    ));
                }
                Some(trimmed)
            }
            _ => None,
        };

        // Check entry exists
        let entry_result = game_entries::find_entry_by_id(self.db, entry_id)?;
        if entry_result.is_none() {
            return Err(AppError::EntryNotFound(entry_id));
        }

        // Update
        game_entries::update_source(self.db, entry_id, value)?;
        Ok(())
    }

    pub fn update_last_played(&self, entry_id: i64, date: Option<&str>) -> Result<(), AppError> {
        // Validate date
        let value = match date {
            Some(d) if !d.trim().is_empty() => {
                let parsed = NaiveDate::parse_from_str(d.trim(), "%Y-%m-%d")?;

                if parsed > Local::now().date_naive() {
                    return Err(AppError::InvalidDate(
                        "Invalid date. Must be preset or past date".to_string(),
                    ));
                }
                Some(parsed.format("%Y-%m-%d").to_string())
            }
            _ => None,
        };

        // Check entry exists
        let entry_result = game_entries::find_entry_by_id(self.db, entry_id)?;
        if entry_result.is_none() {
            return Err(AppError::EntryNotFound(entry_id));
        }

        // Update
        game_entries::update_last_played(self.db, entry_id, value.as_deref())?;
        Ok(())
    }
}
