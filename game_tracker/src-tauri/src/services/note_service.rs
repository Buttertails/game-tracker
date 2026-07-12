use crate::{
    db::{notes, Database},
    models::{error::AppError, TimestampedNote},
};
pub struct NoteService<'a> {
    db: &'a Database,
}

impl<'a> NoteService<'a> {
    pub fn new(db: &'a Database) -> Self {
        NoteService { db }
    }

    pub fn add(&self, entry_id: i64, text: &str) -> Result<TimestampedNote, AppError> {
        let trimmed = text.trim();

        if trimmed.is_empty() {
            return Err(AppError::ValidationError(
                "Note can not be empty".to_string(),
            ));
        }

        if trimmed.len() > 2000 {
            return Err(AppError::ValidationError(
                "Must be 2000 characters or less".to_string(),
            ));
        }

        let note = notes::insert_note(self.db, entry_id, trimmed)?;
        Ok(note)
    }

    pub fn delete(&self, note_id: i64) -> Result<(), AppError> {
        let result = notes::delete_note(self.db, note_id)?;

        if result == 0 {
            return Err(AppError::NoteNotFound("Note not found".to_string()));
        }

        Ok(())
    }

    pub fn list_by_entry(&self, entry_id: i64) -> Result<Vec<TimestampedNote>, AppError> {
        let notes = notes::list_notes_by_entry(self.db, entry_id)?;

        Ok(notes)
    }
}
