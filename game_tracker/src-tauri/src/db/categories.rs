use super::Database;
use crate::models::Category;

use rusqlite::{params, OptionalExtension, Result};

pub fn insert_category(
    db: &Database,
    name: &str,
    is_preset: bool,
    display_order: i32,
) -> Result<Category> {
    let mut stmt = db.conn.prepare(
        "INSERT INTO categories (name, is_preset, display_order) VALUES (?1, ?2, ?3) RETURNING *",
    )?;

    let category = stmt.query_row(params![name, is_preset, display_order], |row| {
        Ok(Category {
            category_id: row.get(0)?,
            name: row.get(1)?,
            is_preset: row.get(2)?,
            display_order: row.get(3)?,
            entry_count: 0,
        })
    })?;

    Ok(category)
}

pub fn get_all_categories(db: &Database) -> Result<Vec<Category>> {
    let mut stmt = db.conn.prepare(
        "SELECT c.category_id, c.name, c.is_preset, c.display_order, 
        COUNT(g.category_id) AS entries_count 
        FROM categories c LEFT JOIN game_entries g ON c.category_id = g.category_id 
        GROUP BY c.category_id, c.name, c.is_preset, c.display_order
        ORDER BY is_preset DESC, display_order ASC, c.name ASC",
    )?;

    let cats: Vec<Category> = stmt
        .query_map([], |row| {
            Ok(Category {
                category_id: row.get(0)?,
                name: row.get(1)?,
                is_preset: row.get(2)?,
                display_order: row.get(3)?,
                entry_count: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;

    Ok(cats)
}

pub fn delete_category(db: &Database, id: i64) -> Result<usize> {
    let result = db
        .conn
        .execute("DELETE FROM categories WHERE category_id = ?1", [id])?;

    Ok(result)
}

pub fn update_category_name(db: &Database, id: i64, new_name: &str) -> Result<Category> {
    let mut stmt = db
        .conn
        .prepare("UPDATE categories SET name = ?1 WHERE category_id = ?2 RETURNING *")?;

    let result = stmt.query_row(params![new_name, id], |row| {
        Ok(Category {
            category_id: row.get(0)?,
            name: row.get(1)?,
            is_preset: row.get(2)?,
            display_order: row.get(3)?,
            entry_count: 0,
        })
    })?;

    Ok(result)
}

pub fn find_category_by_name(db: &Database, name: &str) -> Result<Option<Category>> {
    let mut stmt = db
        .conn
        .prepare("SELECT * FROM categories WHERE name = ?1 COLLATE NOCASE")?;

    let result = stmt
        .query_row([name], |row| {
            Ok(Category {
                category_id: row.get(0)?,
                name: row.get(1)?,
                is_preset: row.get(2)?,
                display_order: row.get(3)?,
                entry_count: 0,
            })
        })
        .optional()?;

    Ok(result)
}

pub fn find_category_by_id(db: &Database, id: i64) -> Result<Option<Category>> {
    let mut stmt = db
        .conn
        .prepare("SELECT * FROM categories WHERE category_id = ?1")?;

    let result = stmt
        .query_row([id], |row| {
            Ok(Category {
                category_id: row.get(0)?,
                name: row.get(1)?,
                is_preset: row.get(2)?,
                display_order: row.get(3)?,
                entry_count: 0,
            })
        })
        .optional()?;

    Ok(result)
}

pub fn get_category_entry_count(db: &Database, id: i64) -> Result<i64> {
    let mut stmt = db
        .conn
        .prepare("SELECT COUNT(*) FROM game_entries WHERE category_id = ?1")?;

    let result = stmt.query_row([id], |row| row.get(0))?;

    Ok(result)
}
