use super::notes::list_notes_by_entry;
use super::tags::get_tags_for_entry;
use super::Database;
use crate::models::{
    format_duration, GameEntry, GameEntryDetail, GameStatus, LengthCategory, OwnershipStatus,
    Shelf, ShelfEntries, TimestampedNote,
};

use rusqlite::{params, Result};

pub fn insert_entry(
    db: &Database,
    shelf_id: i64,
    name: &str,
    genre: Option<&str>,
    length_category: Option<i64>,
    release_year: Option<i32>,
    era: Option<i64>,
    source: Option<&str>,
    launch_path: Option<&str>,
    ownership_status: i64,
    stored_api_data: Option<serde_json::Value>,
) -> Result<i64> {
    let mut stmt = db.conn.prepare(
    "INSERT INTO 
            game_entries (shelf_id, name, genre, length_category, release_year, era, source, launch_path, ownership_status, stored_api_data) 
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10) 
        RETURNING entry_id"
    )?;

    let entry_id = stmt.query_row(
        params![
            shelf_id,
            name,
            genre,
            length_category,
            release_year,
            era,
            source,
            launch_path,
            ownership_status,
            stored_api_data
        ],
        |row| row.get(0),
    )?;

    Ok(entry_id)
}

pub fn list_entries_by_shelf(db: &Database, id: i64) -> Result<ShelfEntries> {
    let mut stmt = db
        .conn
        .prepare("SELECT * FROM shelves WHERE shelf_id = ?1")?;

    let shelf = stmt.query_row([id], |row| {
        Ok(Shelf {
            shelf_id: row.get(0)?,
            name: row.get(1)?,
            created_at: row.get(2)?,
        })
    })?;

    stmt = db
        .conn
        .prepare("SELECT * FROM game_entries WHERE shelf_id = ?1")?;
    let mut shelf_entries = stmt
        .query_map([id], |row| {
            Ok(GameEntry {
                entry_id: row.get(0)?,
                shelf_id: row.get(1)?,
                name: row.get(2)?,
                status: row
                    .get::<_, Option<i64>>(3)?
                    .and_then(GameStatus::from_id)
                    .unwrap_or(GameStatus::Backlog),
                genre: row.get(4)?,
                length_category: row
                    .get::<_, Option<i64>>(5)?
                    .and_then(LengthCategory::from_id),
                release_year: row.get(6)?,
                source: row.get(8)?,
                launch_path: row.get(9)?,
                ownership_status: row
                    .get::<_, Option<i64>>(10)?
                    .and_then(OwnershipStatus::from_id)
                    .unwrap_or(OwnershipStatus::Wishlisted),
                started_at: row.get(11)?,
                completed_at: row.get(12)?,
                addition_date: row.get(13)?,
                tags: Vec::<String>::new(),
                last_played: row.get(14)?,
            })
        })?
        .collect::<Result<Vec<GameEntry>>>()?;

    for entry in &mut shelf_entries {
        entry.tags = get_tags_for_entry(db, entry.entry_id)?;
    }

    Ok(ShelfEntries {
        shelf: shelf,
        backlog: shelf_entries
            .iter()
            .filter(|g| g.status == GameStatus::Backlog)
            .map(|g| g.clone())
            .collect::<Vec<GameEntry>>(),
        in_progress: shelf_entries
            .iter()
            .filter(|g| g.status == GameStatus::InProgress)
            .map(|g| g.clone())
            .collect::<Vec<GameEntry>>(),
        completed: shelf_entries
            .iter()
            .filter(|g| g.status == GameStatus::Completed)
            .map(|g| g.clone())
            .collect::<Vec<GameEntry>>(),
    })
}

pub fn get_entry(db: &Database, id: i64) -> Result<GameEntry> {
    let mut stmt = db
        .conn
        .prepare("SELECT * FROM game_entries WHERE entry_id = ?1")?;

    let mut detail = stmt.query_row([id], |row| {
        Ok(GameEntry {
            entry_id: row.get(0)?,
            shelf_id: row.get(1)?,
            name: row.get(2)?,
            status: row
                .get::<_, Option<i64>>(3)?
                .and_then(GameStatus::from_id)
                .unwrap_or(GameStatus::Backlog),
            genre: row.get(4)?,
            length_category: row
                .get::<_, Option<i64>>(5)?
                .and_then(LengthCategory::from_id),
            release_year: row.get(6)?,
            source: row.get(8)?,
            launch_path: row.get(9)?,
            ownership_status: row
                .get::<_, Option<i64>>(10)?
                .and_then(OwnershipStatus::from_id)
                .unwrap_or(OwnershipStatus::Wishlisted),
            started_at: row.get(11)?,
            completed_at: row.get(12)?,
            addition_date: row.get(13)?,
            tags: Vec::<String>::new(),
            last_played: row.get(14)?,
        })
    })?;

    detail.tags = get_tags_for_entry(db, id)?;

    Ok(detail)
}

pub fn get_entry_detail(db: &Database, id: i64) -> Result<GameEntryDetail> {
    let mut stmt = db
        .conn
        .prepare("SELECT * FROM game_entries WHERE entry_id = ?1")?;

    let mut detail = stmt.query_row([id], |row| {
        Ok(GameEntryDetail {
            entry_id: row.get(0)?,
            shelf_id: row.get(1)?,
            name: row.get(2)?,
            status: row
                .get::<_, Option<i64>>(3)?
                .and_then(GameStatus::from_id)
                .unwrap_or(GameStatus::Backlog),
            genre: row.get(4)?,
            length_category: row
                .get::<_, Option<i64>>(5)?
                .and_then(LengthCategory::from_id),
            release_year: row.get(6)?,
            source: row.get(8)?,
            launch_path: row.get(9)?,
            ownership_status: row
                .get::<_, Option<i64>>(10)?
                .and_then(OwnershipStatus::from_id)
                .unwrap_or(OwnershipStatus::Wishlisted),
            started_at: row.get(11)?,
            completed_at: row.get(12)?,
            addition_date: row.get(13)?,
            tags: Vec::<String>::new(),
            last_played: row.get(14)?,
            notes: Vec::<TimestampedNote>::new(),
            stored_api_data: row.get(15)?,
            completion_duration: None,
        })
    })?;

    detail.tags = get_tags_for_entry(db, id)?;
    detail.notes = list_notes_by_entry(db, id)?;
    detail.completion_duration = match (&detail.started_at, &detail.completed_at) {
        (Some(start), Some(end)) => Some(format_duration(*start, *end)),
        _ => None,
    };

    Ok(detail)
}

pub fn delete_entry(db: &Database, id: i64) -> Result<usize> {
    let result = db
        .conn
        .execute("DELETE FROM game_entries WHERE entry_id = ?1", [id])?;

    Ok(result)
}

pub fn update_status(db: &Database, id: i64, new_status: GameStatus) -> Result<usize> {
    let result = db.conn.execute(
        "UPDATE game_entries SET status = ?1 WHERE entry_id = ?2",
        params![new_status.to_id(), id],
    )?;

    Ok(result)
}

pub fn update_timestamps(
    db: &Database,
    id: i64,
    started_at: Option<&str>,
    completed_at: Option<&str>,
) -> Result<usize> {
    let result = db.conn.execute(
        "UPDATE game_entries SET started_at = ?1, completed_at = ?2 WHERE entry_id = ?3",
        params![started_at, completed_at, id],
    )?;

    Ok(result)
}

pub fn update_ownership_status(
    db: &Database,
    id: i64,
    ownership_status: OwnershipStatus,
) -> Result<usize> {
    let result = db.conn.execute(
        "UPDATE game_entries SET ownership_status = ?1 WHERE entry_id = ?2",
        params![ownership_status.to_id(), id],
    )?;

    Ok(result)
}

pub fn update_launch_path(db: &Database, id: i64, new_path: Option<&str>) -> Result<usize> {
    let result = db.conn.execute(
        "UPDATE game_entries SET launch_path = ?1 WHERE entry_id = ?2",
        params![new_path, id],
    )?;

    Ok(result)
}

pub fn update_entry_metadata(db: &Database, id: i64, metadata: &str) -> Result<usize> {
    let result = db.conn.execute(
        "UPDATE game_entries SET stored_api_data = ?1 WHERE entry_id = ?2",
        params![metadata, id],
    )?;

    Ok(result)
}

pub fn count_in_progress(db: &Database, id: i64) -> Result<i64> {
    let mut stmt = db
        .conn
        .prepare("SELECT count(status) FROM game_entries WHERE status = ?1 and shelf_id = ?2")?;

    let result = stmt.query_row(
        params![GameStatus::to_id(&GameStatus::InProgress), id],
        |row| row.get(0),
    )?;

    Ok(result)
}
