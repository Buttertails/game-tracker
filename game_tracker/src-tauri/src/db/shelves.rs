use super::Database;
use crate::models::{Shelf, ShelfSummary};

use rusqlite::{params, OptionalExtension, Result};

pub fn insert_shelf(db: &Database, name: &str) -> Result<Shelf> {
    let mut stmt = db
        .conn
        .prepare("INSERT INTO shelves (name) VALUES (?1) RETURNING *")?;

    let shelf = stmt.query_row([name], |row| {
        Ok(Shelf {
            shelf_id: row.get(0)?,
            name: row.get(1)?,
            created_at: row.get(2)?,
        })
    })?;

    Ok(shelf)
}

pub fn get_shelf(db: &Database, id: i64) -> Result<Shelf> {
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

    Ok(shelf)
}

pub fn find_shelf_by_name(db: &Database, name: &str) -> Result<Option<Shelf>> {
    let mut stmt = db
        .conn
        .prepare("SELECT * FROM shelves WHERE name = ?1 COLLATE NOCASE")?;

    stmt.query_row([name], |row| {
        Ok(Shelf {
            shelf_id: row.get(0)?,
            name: row.get(1)?,
            created_at: row.get(2)?,
        })
    })
    .optional()
}

pub fn rename_shelf(db: &Database, id: i64, new_name: &str) -> Result<Shelf> {
    let mut stmt = db
        .conn
        .prepare("UPDATE shelves SET name = ?1 WHERE shelf_id = ?2 RETURNING *")?;

    let shelf = stmt.query_row(params![new_name, id], |row| {
        Ok(Shelf {
            shelf_id: row.get(0)?,
            name: row.get(1)?,
            created_at: row.get(2)?,
        })
    })?;

    Ok(shelf)
}

pub fn delete_shelf(db: &Database, id: i64) -> Result<usize> {
    let result = db
        .conn
        .execute("DELETE FROM shelves WHERE shelf_id = ?1", [id])?;

    Ok(result)
}

pub fn list_shelves_with_counts(db: &Database) -> Result<Vec<ShelfSummary>> {
    let mut stmt = db.conn.prepare(
        "SELECT 
	        s.shelf_id,
	        s.name, 
	        count(CASE WHEN g.status = 1 THEN 1 END) as backlog_count,
	        count(CASE WHEN g.status = 2 THEN 1 END) as in_progress_count,
	        count(CASE WHEN g.status = 3 THEN 1 END) as completed_count
        FROM shelves as s 
        LEFT JOIN game_entries as g ON s.shelf_id = g.shelf_id
        GROUP BY s.shelf_id, s.name
        ORDER BY s.shelf_id ASC",
    )?;

    let shelves: Vec<ShelfSummary> = stmt
        .query_map([], |row| {
            Ok(ShelfSummary {
                shelf_id: row.get(0)?,
                name: row.get(1)?,
                backlog_count: row.get(2)?,
                in_progress_count: row.get(3)?,
                completed_count: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;

    Ok(shelves)
}
