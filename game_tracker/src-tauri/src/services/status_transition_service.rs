use crate::db::{game_entries, Database};
use crate::models::error::AppError::{self};
use crate::models::GameStatus;
use chrono::Utc;

pub struct StatusTransitionService<'a> {
    db: &'a Database,
}

impl<'a> StatusTransitionService<'a> {
    pub fn new(db: &'a Database) -> Self {
        StatusTransitionService { db }
    }

    pub fn move_to_in_progress(&self, entry_id: i64) -> Result<(), AppError> {
        let entry = game_entries::get_entry(self.db, entry_id)?;

        if entry.status != GameStatus::Backlog {
            return Err(AppError::InvalidTransition {
                from: entry.status.to_string(),
                to: GameStatus::InProgress.to_string(),
                reason: "Can only move from Backlog to In Progress".to_string(),
            });
        }

        let in_progress_count = game_entries::count_in_progress(self.db, entry.shelf_id)?;
        if in_progress_count >= 3 {
            return Err(AppError::InProgressFull {
                shelf_id: entry.shelf_id,
                cap: 3,
            });
        }

        game_entries::update_status(self.db, entry_id, GameStatus::InProgress)?;
        game_entries::update_started_at(
            self.db,
            entry_id,
            Some(&Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()),
        )?;

        Ok(())
    }

    pub fn move_to_completed(&self, entry_id: i64) -> Result<(), AppError> {
        let entry = game_entries::get_entry(self.db, entry_id)?;

        if entry.status != GameStatus::InProgress {
            return Err(AppError::InvalidTransition {
                from: entry.status.to_string(),
                to: GameStatus::Completed.to_string(),
                reason: "Can only move from In Progress to Completed".to_string(),
            });
        }

        game_entries::update_status(self.db, entry_id, GameStatus::Completed)?;
        game_entries::update_completed_at(
            self.db,
            entry_id,
            Some(&Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()),
        )?;

        Ok(())
    }

    pub fn move_to_backlog(&self, entry_id: i64) -> Result<(), AppError> {
        let entry = game_entries::get_entry(self.db, entry_id)?;

        if entry.status != GameStatus::InProgress {
            return Err(AppError::InvalidTransition {
                from: entry.status.to_string(),
                to: GameStatus::Backlog.to_string(),
                reason: "Can only move from InProgress to Backlog".to_string(),
            });
        }

        game_entries::update_status(self.db, entry_id, GameStatus::Backlog)?;
        game_entries::update_started_at(self.db, entry_id, None)?;

        Ok(())
    }

    pub fn undo_to_in_progress(&self, entry_id: i64, confirmed: bool) -> Result<(), AppError> {
        if !confirmed {
            return Err(AppError::ConfirmationRequired("Must confirm".to_string()));
        }

        let entry = game_entries::get_entry(self.db, entry_id)?;

        if entry.status != GameStatus::Completed {
            return Err(AppError::InvalidTransition {
                from: entry.status.to_string(),
                to: GameStatus::InProgress.to_string(),
                reason: "Can only undo from Completed".to_string(),
            });
        }

        let in_progress_count = game_entries::count_in_progress(self.db, entry.shelf_id)?;
        if in_progress_count >= 3 {
            return Err(AppError::InProgressFull {
                shelf_id: entry.shelf_id,
                cap: 3,
            });
        }

        game_entries::update_status(self.db, entry_id, GameStatus::InProgress)?;
        game_entries::update_started_at(
            self.db,
            entry_id,
            Some(&Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()),
        )?;
        game_entries::update_completed_at(self.db, entry_id, None)?;

        Ok(())
    }

    pub fn undo_to_backlog(&self, entry_id: i64, confirmed: bool) -> Result<(), AppError> {
        if !confirmed {
            return Err(AppError::ConfirmationRequired("Must confirm".to_string()));
        }

        let entry = game_entries::get_entry(self.db, entry_id)?;

        if entry.status != GameStatus::Completed {
            return Err(AppError::InvalidTransition {
                from: entry.status.to_string(),
                to: GameStatus::Backlog.to_string(),
                reason: "Can only undo from Completed".to_string(),
            });
        }

        game_entries::update_status(self.db, entry_id, GameStatus::Backlog)?;
        game_entries::update_started_at(self.db, entry_id, None)?;
        game_entries::update_completed_at(self.db, entry_id, None)?;
        Ok(())
    }
}
