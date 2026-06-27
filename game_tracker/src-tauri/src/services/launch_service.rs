use crate::{
    db::{game_entries, Database},
    models::error::AppError,
};
use std::path::Path;
use std::process::Command;
pub struct GameLaunchService<'a> {
    db: &'a Database,
}

impl<'a> GameLaunchService<'a> {
    pub fn new(db: &'a Database) -> Self {
        GameLaunchService { db }
    }

    pub fn validate_path(&self, path: &str) -> Result<(), AppError> {
        let path_exists = Path::new(path).exists();

        if !path_exists {
            return Err(AppError::LaunchPathNotFound("Path not found".to_string()));
        }

        Ok(())
    }

    pub fn set_path(&self, entry_id: i64, path: Option<&str>) -> Result<(), AppError> {
        let value = match path {
            Some(p) if !p.trim().is_empty() => {
                let trimmed_path = p.trim();
                self.validate_path(trimmed_path)?;

                Some(trimmed_path)
            }
            _ => None,
        };

        // Check entry exists
        let entry_result = game_entries::find_entry_by_id(self.db, entry_id)?;
        if entry_result.is_none() {
            return Err(AppError::EntryNotFound(entry_id));
        }

        game_entries::update_launch_path(self.db, entry_id, value)?;

        Ok(())
    }

    pub fn launch(&self, entry_id: i64) -> Result<(), AppError> {
        let entry_detail = game_entries::get_game_entry_detail(self.db, entry_id)?;

        let path = entry_detail.launch_path.ok_or(AppError::NoLaunchPath)?;
        let trimmed = path.trim();

        if trimmed.is_empty() {
            return Err(AppError::NoLaunchPath);
        }

        self.validate_path(trimmed)?;

        Command::new(trimmed).spawn()?;
        Ok(())
    }
}
