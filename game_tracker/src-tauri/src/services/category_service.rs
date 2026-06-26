use crate::db::{categories, Database};
use crate::models::error::AppError::{self, ValidationError};
use crate::models::Category;
pub struct CategoryService<'a> {
    db: &'a Database,
}

impl<'a> CategoryService<'a> {
    pub fn new(db: &'a Database) -> Self {
        CategoryService { db }
    }

    pub fn initialize_presets(&self) -> Result<(), AppError> {
        let presets = vec![
            ("Not Started", 1),
            ("Started", 2),
            ("In Progress", 3),
            ("Completed", 4),
        ];

        for (name, order) in presets {
            if categories::find_category_by_name(self.db, name)?.is_none() {
                categories::insert_category(self.db, name, true, order)?;
            }
        }

        Ok(())
    }

    pub fn create(&self, name: &str) -> Result<Category, AppError> {
        let trimmed = name.trim();

        if trimmed.is_empty() {
            return Err(AppError::ValidationError(
                "A valid name is required".to_string(),
            ));
        }

        if trimmed.len() > 50 {
            return Err(AppError::ValidationError(
                "Category name must be 50 characters or less".to_string(),
            ));
        }

        if categories::find_category_by_name(self.db, trimmed)?.is_some() {
            return Err(AppError::DuplicateCategory(
                "Category name is already in use".to_string(),
            ));
        }

        let all = categories::get_all_categories(self.db)?;
        let next_order = all.iter().map(|c| c.display_order).max().unwrap_or(0) + 1;

        let category = categories::insert_category(self.db, trimmed, false, next_order)?;
        Ok(category)
    }

    pub fn delete(&self, id: i64, confirm_with_entries: bool) -> Result<(), AppError> {
        let entry_count = categories::get_category_entry_count(self.db, id)?;

        if entry_count != 0 && !confirm_with_entries {
            return Err(AppError::CategoryNotEmpty {
                category_id: id,
                entry_count: entry_count,
            });
        }

        let deleted = categories::delete_category(self.db, id)?;
        if deleted == 0 {
            return Err(AppError::CategoryNotFound(id));
        }

        Ok(())
    }

    pub fn list_all(&self) -> Result<Vec<Category>, AppError> {
        let cats = categories::get_all_categories(self.db)?;

        Ok(cats)
    }
}
