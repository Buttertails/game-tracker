use super::tags::get_tags_for_entry;
use super::Database;
use crate::models::DuplicateInfo;
use crate::models::GameEntry;
use crate::models::GameEntryDetail;

use rusqlite::OptionalExtension;
use rusqlite::{params, Result};

pub fn insert_game_entry(
    db: &Database,
    name: &str,
    category_id: i64,
    source: Option<&str>,
    stored_api_data: Option<serde_json::Value>,
) -> Result<i64> {
    let mut stmt = db.conn.prepare("INSERT INTO game_entries (name, category_id, source, stored_api_data) VALUES (?1, ?2, ?3, ?4) RETURNING game_entry_id")?;

    let game_entry_id = stmt
        .query_row(params![name, category_id, source, stored_api_data], |row| {
            row.get(0)
        })?;

    Ok(game_entry_id)
}

pub fn get_entries_by_category(db: &Database, category_id: i64) -> Result<Vec<GameEntry>> {
    let mut stmt = db.conn.prepare(
        "SELECT g.game_entry_id, g.name, g.category_id, c.name, g.addition_date, g.source, g.last_played
            FROM game_entries g JOIN categories c ON g.category_id = c.category_id 
            WHERE c.category_id = ?1")?;

    let results = stmt.query_map([category_id], |row| {
        Ok(GameEntry {
            entry_id: row.get(0)?,
            name: row.get(1)?,
            category_id: row.get(2)?,
            category_name: row.get(3)?,
            addition_date: row.get(4)?,
            tags: Vec::<String>::new(),
            source: row.get(5)?,
            last_played: row.get(6)?,
        })
    })?;

    let mut entries = Vec::<GameEntry>::new();

    for result in results {
        let mut entry = result?;
        entry.tags = get_tags_for_entry(db, entry.entry_id)?;
        entries.push(entry);
    }

    Ok(entries)
}

pub fn get_game_entry_detail(db: &Database, id: i64) -> Result<GameEntryDetail> {
    let mut stmt = db.conn.prepare(
        "SELECT g.game_entry_id, g.name, g.category_id, c.name, g.addition_date, g.source, g.last_played, g.launch_path, g.stored_api_data
            FROM game_entries g JOIN categories c on g.category_id = c.category_id
            WHERE game_entry_id = ?1")?;

    let mut detail = stmt.query_row([id], |row| {
        Ok(GameEntryDetail {
            entry_id: row.get(0)?,
            name: row.get(1)?,
            category_id: row.get(2)?,
            category_name: row.get(3)?,
            addition_date: row.get(4)?,
            tags: Vec::<String>::new(),
            source: row.get(5)?,
            last_played: row.get(6)?,
            launch_path: row.get(7)?,
            stored_api_data: row.get(8)?,
        })
    })?;

    detail.tags = get_tags_for_entry(db, id)?;

    Ok(detail)
}

pub fn find_entry_by_name(db: &Database, name: &str) -> Result<Option<DuplicateInfo>> {
    let mut stmt = db.conn.prepare(
        "
        SELECT g.game_entry_id, g.name, c.name 
        FROM game_entries g JOIN categories c ON g.category_id = c.category_id
        WHERE g.name = ?1 COLLATE NOCASE",
    )?;

    let result = stmt
        .query_row([name], |row| {
            Ok(DuplicateInfo {
                entry_id: row.get(0)?,
                name: row.get(1)?,
                category_name: row.get(2)?,
            })
        })
        .optional()?;

    Ok(result)
}

pub fn update_game_entry_category(
    db: &Database,
    id: i64,
    new_category_id: i64,
) -> Result<GameEntry> {
    let mut stmt = db.conn.prepare(
        "UPDATE game_entries 
        SET category_id = ?1 
        WHERE game_entry_id = ?2
        RETURNING game_entry_id, name, category_id,
            (SELECT name FROM categories WHERE category_id = game_entries.category_id) as category_name,
            addition_date, source, last_played",
    )?;

    let mut result = stmt.query_row([new_category_id, id], |row| {
        Ok(GameEntry {
            entry_id: row.get(0)?,
            name: row.get(1)?,
            category_id: row.get(2)?,
            category_name: row.get(3)?,
            addition_date: row.get(4)?,
            tags: Vec::<String>::new(),
            source: row.get(5)?,
            last_played: row.get(6)?,
        })
    })?;

    result.tags = get_tags_for_entry(db, id)?;

    Ok(result)
}

pub fn update_launch_path(db: &Database, id: i64, path: Option<&str>) -> Result<usize> {
    let result = db.conn.execute(
        "UPDATE game_entries 
        SET launch_path = ?1
        WHERE game_entry_id = ?2",
        params![path, id],
    )?;

    Ok(result)
}

pub fn update_last_played(db: &Database, id: i64, date: Option<&str>) -> Result<usize> {
    let result = db.conn.execute(
        "UPDATE game_entries 
        SET last_played = ?1
        WHERE game_entry_id = ?2",
        params![date, id],
    )?;

    Ok(result)
}

pub fn update_source(db: &Database, id: i64, source: Option<&str>) -> Result<usize> {
    let result = db.conn.execute(
        "UPDATE game_entries 
        SET source = ?1
        WHERE game_entry_id = ?2",
        params![source, id],
    )?;

    Ok(result)
}

pub fn delete_game_entry(db: &Database, id: i64) -> Result<usize> {
    let result = db
        .conn
        .execute("DELETE FROM game_entries WHERE game_entry_id = ?1", [id])?;

    Ok(result)
}
