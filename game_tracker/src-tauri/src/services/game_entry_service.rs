use tauri::App;

use crate::db::{game_entries, shelves, tags, Database};
use crate::models::error::AppError;
use crate::models::{
    derive_era, derive_length_category, GameEntryDetail, GameStatus, ManualEntryInput,
    OwnershipStatus, RawgGameData, ShelfEntries,
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
        rawg_data: &RawgGameData,
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

        // Extract genre from RAWG data
        let genre = rawg_data
            .genres
            .as_ref()
            .and_then(|g| g.first())
            .map(|g| g.name.clone());

        // Extract length category from RAWG data
        let playtime = rawg_data
            .extra
            .get("playtime")
            .and_then(|v| v.as_i64())
            .filter(|&p| p > 0)
            .map(|p| derive_length_category(p).to_id());

        // Extract release date and parse release year
        let release_year = rawg_data
            .released
            .as_ref()
            .and_then(|r| r.split('-').next()?.parse::<i32>().ok());

        // Derive era from release year
        let era = release_year.map(|r| derive_era(r).to_id());

        // Extract extra json data from RAWG data
        let rawg_data_json = Some(serde_json::to_value(&rawg_data)?);

        let entry_id = game_entries::insert_entry(
            self.db,
            shelf_id,
            rawg_data.name.as_str(),
            GameStatus::Backlog.to_id(),
            genre.as_deref(),
            playtime,
            release_year,
            era,
            source,
            launch_path,
            ownership_status.to_id(),
            rawg_data_json,
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
        if let Some(year) = input.release_year {
            if year < 1950 || year > 2100 {
                return Err(AppError::ValidationError(
                    "Release year must be between 1950 and 2100".to_string(),
                ));
            }
        }
        let era_id = input.release_year.map(|y| derive_era(y).to_id());

        // Validate optional length category
        let length_category_id = input.length_category.map(|lc| lc.to_id());

        // Validate shelf exists
        let shelf = shelves::get_shelf(self.db, input.shelf_id);
        if shelf.is_err() {
            return Err(AppError::ShelfNotFound(input.shelf_id));
        }

        // Insert game entry w/o tags
        let entry_id = game_entries::insert_entry(
            self.db,
            input.shelf_id,
            trimmed,
            GameStatus::Backlog.to_id(),
            input.genre.as_deref(),
            length_category_id,
            input.release_year,
            era_id,
            input.source.as_deref(),
            input.launch_path.as_deref(),
            input.ownership_status.to_id(),
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
}
