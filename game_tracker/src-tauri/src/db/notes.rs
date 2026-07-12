use crate::models::TimestampedNote;

use rusqlite::{params, Result};

use super::Database;

pub fn insert_note(db: &Database, id: i64, note: &str) -> Result<TimestampedNote> {
    let mut stmt = db
        .conn
        .prepare("INSERT INTO game_notes(entry_id, text) VALUES (?1, ?2) RETURNING *")?;

    let note = stmt.query_row(params![id, note], |row| {
        Ok(TimestampedNote {
            note_id: row.get(0)?,
            entry_id: row.get(1)?,
            text: row.get(2)?,
            created_at: row.get(3)?,
        })
    })?;

    Ok(note)
}

pub fn delete_note(db: &Database, id: i64) -> Result<usize> {
    let result = db
        .conn
        .execute("DELETE FROM game_notes WHERE note_id = ?1", [id])?;

    Ok(result)
}

pub fn list_notes_by_entry(db: &Database, id: i64) -> Result<Vec<TimestampedNote>> {
    let mut stmt = db
        .conn
        .prepare("SELECT * FROM game_notes WHERE entry_id = ?1 ORDER BY created_at DESC")?;

    let notes = stmt
        .query_map([id], |row| {
            Ok(TimestampedNote {
                note_id: row.get(0)?,
                entry_id: row.get(1)?,
                text: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?
        .collect::<Result<Vec<TimestampedNote>>>();

    notes
}
