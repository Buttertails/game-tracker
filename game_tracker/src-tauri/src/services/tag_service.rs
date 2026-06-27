use crate::{
    db::{game_entries, tags, Database},
    models::error::AppError,
};
pub struct TagService<'a> {
    db: &'a Database,
}

impl<'a> TagService<'a> {
    pub fn new(db: &'a Database) -> Self {
        TagService { db }
    }

    pub fn add(&self, entry_id: i64, tag: &str) -> Result<(), AppError> {
        // Validate tag
        let trimmed = tag.trim();

        // Validate non-empty
        if trimmed.is_empty() {
            return Err(AppError::ValidationError(
                "A valid name is required".to_string(),
            ));
        }

        // Validate tag char length
        if trimmed.len() > 30 {
            return Err(AppError::ValidationError(
                "Tag must be 30 characters or less".to_string(),
            ));
        }

        // Check if entry exists
        let result = game_entries::find_entry_by_id(self.db, entry_id)?;
        if result.is_none() {
            return Err(AppError::EntryNotFound(entry_id));
        }

        // Check if duplicate tag exists
        let duplicate_tag = tags::tag_exists(self.db, entry_id, trimmed)?;
        if duplicate_tag {
            return Err(AppError::DuplicateTag(trimmed.to_string()));
        }

        // Insert tag, check for successful insert
        let tag_result = tags::insert_tag(self.db, entry_id, trimmed)?;
        if tag_result == 0 {
            return Err(AppError::DatabaseError("Failed to add tag".to_string()));
        }

        Ok(())
    }

    pub fn remove(&self, entry_id: i64, tag: &str) -> Result<(), AppError> {
        // Validate tag
        let trimmed = tag.trim();

        // Validate non-empty
        if trimmed.is_empty() {
            return Err(AppError::ValidationError(
                "A valid name is required".to_string(),
            ));
        }

        // Check if entry exists
        let result = game_entries::find_entry_by_id(self.db, entry_id)?;
        if result.is_none() {
            return Err(AppError::EntryNotFound(entry_id));
        }

        // Check if tag exists
        let tag_exists = tags::tag_exists(self.db, entry_id, trimmed)?;
        if !tag_exists {
            return Err(AppError::TagNotFound("Tag does not exist".to_string()));
        }

        // Delete tag, check for successful delete
        let tag_result = tags::delete_tag(self.db, entry_id, trimmed)?;

        if tag_result == 0 {
            return Err(AppError::DatabaseError("Failed to delete tag".to_string()));
        }

        Ok(())
    }
}
