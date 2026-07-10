use super::Database;

use rusqlite::{params, Result};

pub fn insert_tag(db: &Database, entry_id: i64, tag: &str) -> Result<usize> {
    let result = db.conn.execute(
        "INSERT OR IGNORE INTO game_tags (entry_id, tag) VALUES (?1, ?2)",
        params![entry_id, tag],
    )?;

    Ok(result)
}

pub fn delete_tag(db: &Database, entry_id: i64, tag: &str) -> Result<usize> {
    let result = db.conn.execute(
        "DELETE FROM game_tags 
            WHERE entry_id = ?1 AND tag = ?2 COLLATE NOCASE",
        params![entry_id, tag],
    )?;

    Ok(result)
}

pub fn get_tags_for_entry(db: &Database, entry_id: i64) -> Result<Vec<String>> {
    let mut stmt = db
        .conn
        .prepare("SELECT tag FROM game_tags WHERE entry_id = ?1")?;

    let results: Vec<String> = stmt
        .query_map([entry_id], |row| row.get(0))?
        .collect::<Result<Vec<_>>>()?;

    Ok(results)
}

pub fn tag_exists(db: &Database, entry_id: i64, tag: &str) -> Result<bool> {
    let mut stmt = db.conn.prepare(
        "SELECT EXISTS
        (SELECT 1 FROM game_tags WHERE entry_id = ?1 AND tag = ?2 COLLATE NOCASE)",
    )?;

    let exists = stmt.query_row(params![entry_id, tag], |row| row.get(0))?;

    Ok(exists)
}
