use crate::db::{shelves, Database};
use crate::models::error::AppError::{self};
use crate::models::{Shelf, ShelfSummary};

pub struct ShelfService<'a> {
    db: &'a Database,
}

impl<'a> ShelfService<'a> {
    pub fn new(db: &'a Database) -> Self {
        ShelfService { db }
    }

    pub fn create(&self, name: &str) -> Result<Shelf, AppError> {
        let trimmed = name.trim();

        if trimmed.is_empty() {
            return Err(AppError::ValidationError(
                "A valid name is required".to_string(),
            ));
        }

        if trimmed.len() > 50 {
            return Err(AppError::ValidationError(
                "Shelf name must be 50 characters or less".to_string(),
            ));
        }

        if shelves::find_shelf_by_name(self.db, trimmed)?.is_some() {
            return Err(AppError::DuplicateShelf(
                "Shelf name is already in use".to_string(),
            ));
        }

        let shelf = shelves::insert_shelf(self.db, trimmed)?;
        Ok(shelf)
    }

    pub fn rename(&self, shelf_id: i64, new_name: &str) -> Result<Shelf, AppError> {
        let trimmed = new_name.trim();

        if trimmed.is_empty() {
            return Err(AppError::ValidationError(
                "A valid name is required".to_string(),
            ));
        }

        if trimmed.len() > 50 {
            return Err(AppError::ValidationError(
                "Shelf name must be 50 characters or less".to_string(),
            ));
        }

        if let Some(existing) = shelves::find_shelf_by_name(self.db, trimmed)? {
            if existing.shelf_id != shelf_id {
                return Err(AppError::DuplicateShelf(
                    "Shelf name is already in use".to_string(),
                ));
            }
        }

        let renamed_shelf = shelves::rename_shelf(self.db, shelf_id, trimmed)?;
        Ok(renamed_shelf)
    }

    pub fn delete(&self, shelf_id: i64, confirmed: bool) -> Result<(), AppError> {
        if !confirmed {
            return Err(AppError::ConfirmationRequired(
                "Confirmation required to delete shelf".to_string(),
            ));
        }

        let deleted = shelves::delete_shelf(self.db, shelf_id)?;
        if deleted == 0 {
            return Err(AppError::ShelfNotFound(shelf_id));
        }

        Ok(())
    }

    pub fn list_all(&self) -> Result<Vec<ShelfSummary>, AppError> {
        let shelves = shelves::list_shelves_with_counts(self.db)?;
        Ok(shelves)
    }
}
