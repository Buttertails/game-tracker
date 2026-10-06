use chrono::{Datelike, DateTime, Utc};

use crate::db::{game_entries, shelves, tags, Database};
use crate::models::error::AppError;
use crate::models::{
    GameEntryDetail, GameStatus, 
    ManualEntryInput, OwnershipStatus, SearchResult, ShelfEntries,
};
pub struct GameEntryService<'a> {
    db: &'a Database,
}

impl<'a> GameEntryService<'a> {
    pub fn new(db: &'a Database) -> Self {
        GameEntryService { db }
    }

    pub fn add_from_search(
        &self,
        search_result: &SearchResult,
        shelf_id: i64,
        source: Option<&str>,
        launch_path: Option<&str>,
        ownership_status: OwnershipStatus,
        tag_list: &[String],
    ) -> Result<GameEntryDetail, AppError> {
        // Validate shelf exists
        let shelf = shelves::get_shelf(self.db, shelf_id);
        if shelf.is_err() {
            return Err(AppError::ShelfNotFound(shelf_id));
        }

        let entry_id = game_entries::insert_entry(
            self.db,
            shelf_id,
            Some(search_result.igdb_id),
            search_result.name.as_str(),
            GameStatus::Backlog.to_id(),
            search_result.genre.as_deref(),
            search_result.avg_playtime_hours,
            search_result.release_date,
            source,
            launch_path,
            ownership_status.to_id(),
            Some(search_result.stored_api_data.clone()),
            search_result.background_image.as_deref(),
        )?;

        for tag in tag_list {
            tags::insert_tag(self.db, entry_id, tag)?;
        }

        let game_entry_detail = game_entries::get_entry_detail(self.db, entry_id)?;
        Ok(game_entry_detail)
    }

    pub fn add_manually(
        &self,
        input: ManualEntryInput,
        tag_list: &[String],
    ) -> Result<GameEntryDetail, AppError> {
        let trimmed = input.name.trim();

        // Validate name is non-empty
        if trimmed.is_empty() {
            return Err(AppError::ValidationError(
                "A valid name is required".to_string(),
            ));
        }

        // Validate name char length
        if trimmed.len() > 200 {
            return Err(AppError::ValidationError(
                "Game name must be 200 characters or less".to_string(),
            ));
        }

        // Validate optional release year
        // Derive era into era id if valid
        if let Some(date) = input.release_date {
            let year = date.year();
            if year < 1950 || year > 2100 {
                return Err(AppError::ValidationError(
                    "Release year must be between 1950 and 2100".to_string(),
                ));
            }
        }

        // Validate shelf exists
        let shelf = shelves::get_shelf(self.db, input.shelf_id);
        if shelf.is_err() {
            return Err(AppError::ShelfNotFound(input.shelf_id));
        }

        // Insert game entry w/o tags
        let entry_id = game_entries::insert_entry(
            self.db,
            input.shelf_id,
            None,
            trimmed,
            GameStatus::Backlog.to_id(),
            input.genre.as_deref(),
            input.avg_playtime_hours,
            input.release_date,
            input.source.as_deref(),
            input.launch_path.as_deref(),
            input.ownership_status.to_id(),
            None,
            None,
        )?;

        for tag in tag_list {
            tags::insert_tag(self.db, entry_id, tag)?;
        }

        let entry_detail = game_entries::get_entry_detail(self.db, entry_id)?;
        Ok(entry_detail)
    }

    pub fn get_detail(&self, entry_id: i64) -> Result<GameEntryDetail, AppError> {
        let entry_detail = game_entries::get_entry_detail(self.db, entry_id)?;
        Ok(entry_detail)
    }

    pub fn list_by_shelf(&self, shelf_id: i64) -> Result<ShelfEntries, AppError> {
        let result = game_entries::list_entries_by_shelf(self.db, shelf_id)?;
        Ok(result)
    }

    pub fn delete(&self, entry_id: i64) -> Result<(), AppError> {
        let result = game_entries::delete_entry(self.db, entry_id)?;

        if result == 0 {
            return Err(AppError::EntryNotFound(entry_id));
        };

        Ok(())
    }

    pub fn update_ownership_status(
        &self,
        entry_id: i64,
        new_ownership_status: OwnershipStatus,
    ) -> Result<(), AppError> {
        let result =
            game_entries::update_ownership_status(self.db, entry_id, new_ownership_status)?;

        if result == 0 {
            return Err(AppError::EntryNotFound(entry_id));
        };

        Ok(())
    }

    pub fn update_launch_path(
        &self,
        entry_id: i64,
        new_path: Option<&str>,
    ) -> Result<(), AppError> {
        if let Some(path) = new_path {
            if !std::path::Path::new(path).exists() {
                return Err(AppError::LaunchPathNotFound(path.to_string()));
            }
            game_entries::update_ownership_status(self.db, entry_id, OwnershipStatus::Installed)?;
        }

        let result = game_entries::update_launch_path(self.db, entry_id, new_path)?;
        if result == 0 {
            return Err(AppError::EntryNotFound(entry_id));
        };

        Ok(())
    }

    pub fn update_entry_metadata(
        &self,
        entry_id: i64,
        genre: Option<&str>,
        avg_playtime_hours: Option<i64>,
        release_date: Option<DateTime<Utc>>,
        source: Option<&str>,
    ) -> Result<(), AppError> {
        // validate release year between 1950-2100
        if let Some(date) = release_date {
            let year = date.year();
            if year < 1950 || year > 2100 {
                return Err(AppError::ValidationError(
                    "Release year must be between 1950 and 2100".to_string(),
                ));
            }
        }

        // validate genre non-empty
        if let Some(g) = genre {
            if g.trim().is_empty() {
                return Err(AppError::ValidationError(
                    "A valid genre is required".to_string(),
                ));
            }
        }

        // validate source length 1-100
        if let Some(s) = source {
            if s.trim().is_empty() {
                return Err(AppError::ValidationError(
                    "A valid source is required".to_string(),
                ));
            }

            if s.len() > 100 {
                return Err(AppError::ValidationError(
                    "Source name must be less than 100 characters".to_string(),
                ));
            }
        }

        let result = game_entries::update_entry_metadata(
            self.db,
            entry_id,
            genre,
            avg_playtime_hours,
            release_date,
            source,
        )?;
        if result == 0 {
            return Err(AppError::EntryNotFound(entry_id));
        };

        Ok(())
    }
}
