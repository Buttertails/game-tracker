use crate::db::{categories, game_entries, tags, Database};
use crate::models::error::AppError;
use crate::models::{DuplicateInfo, GameEntry, GameEntryDetail, RawgGameData};
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
        category_id: i64,
        tag_list: &[String],
        source: Option<&str>,
    ) -> Result<GameEntryDetail, AppError> {
        // Validate category exists
        let category = categories::find_category_by_id(self.db, category_id)?;
        if category.is_none() {
            return Err(AppError::CategoryNotFound(category_id));
        }

        let rawg_data_json = Some(serde_json::to_value(rawg_data)?);
        let game_entry_id = game_entries::insert_game_entry(
            self.db,
            rawg_data.name.as_str(),
            category_id,
            source,
            rawg_data_json,
        )?;

        for tag in tag_list {
            tags::insert_tag(self.db, game_entry_id, tag)?;
        }

        let game_entry_detail = game_entries::get_game_entry_detail(self.db, game_entry_id)?;
        Ok(game_entry_detail)
    }

    pub fn add_manually(
        &self,
        name: &str,
        category_id: i64,
        tag_list: &[String],
        source: Option<&str>,
    ) -> Result<GameEntryDetail, AppError> {
        let trimmed = name.trim();

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

        // Validate category exists
        let category = categories::find_category_by_id(self.db, category_id)?;
        if category.is_none() {
            return Err(AppError::CategoryNotFound(category_id));
        }

        // Insert game entry w/o tags
        let game_entry_id =
            game_entries::insert_game_entry(self.db, trimmed, category_id, source, None)?;

        for tag in tag_list {
            tags::insert_tag(self.db, game_entry_id, tag)?;
        }

        let game_entry_detail = game_entries::get_game_entry_detail(self.db, game_entry_id)?;
        Ok(game_entry_detail)
    }

    pub fn move_to_category(
        &self,
        entry_id: i64,
        dest_category_id: i64,
    ) -> Result<GameEntry, AppError> {
        let entry = game_entries::get_game_entry(self.db, entry_id)?;

        if entry.category_id == dest_category_id {
            return Ok(entry);
        }

        // Validate category exists
        let category = categories::find_category_by_id(self.db, dest_category_id)?;
        if category.is_none() {
            return Err(AppError::CategoryNotFound(dest_category_id));
        }

        let result = game_entries::update_game_entry_category(self.db, entry_id, dest_category_id)?;
        Ok(result)
    }

    pub fn delete(&self, entry_id: i64) -> Result<(), AppError> {
        let result = game_entries::delete_game_entry(self.db, entry_id)?;

        if result == 0 {
            return Err(AppError::EntryNotFound(entry_id));
        };

        Ok(())
    }

    pub fn get_detail(&self, entry_id: i64) -> Result<GameEntryDetail, AppError> {
        let result = game_entries::get_game_entry_detail(self.db, entry_id)?;
        Ok(result)
    }

    pub fn list_by_category(&self, category_id: i64) -> Result<Vec<GameEntry>, AppError> {
        let result = game_entries::get_entries_by_category(self.db, category_id)?;
        Ok(result)
    }

    pub fn check_duplicate_by_name(&self, name: &str) -> Result<Option<DuplicateInfo>, AppError> {
        let trimmed = name.trim();
        let result = game_entries::find_entry_by_name(self.db, trimmed)?;
        Ok(result)
    }
}
