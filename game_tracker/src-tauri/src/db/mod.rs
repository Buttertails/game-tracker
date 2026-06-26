pub mod categories;

use rusqlite::{params, Connection, Result};

pub struct Database {
    pub conn: Connection,
}

impl Database {
    pub fn new(path: &str) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(path)?;

        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;

        let db = Database { conn };
        db.run_migrations()?;
        Ok(db)
    }

    pub fn new_in_memory() -> Result<Self, rusqlite::Error> {
        let conn = Connection::open_in_memory()?;

        conn.execute_batch("PRAGMA foreign_keys=ON;")?;

        let db = Database { conn };
        db.run_migrations()?;
        Ok(db)
    }

    fn run_migrations(&self) -> Result<(), rusqlite::Error> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS categories (
                category_id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                is_preset INTEGER NOT NULL DEFAULT 0,
                display_order INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            
            CREATE TABLE IF NOT EXISTS game_entries (
                game_entry_id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                category_id INTEGER NOT NULL,
                addition_date TEXT NOT NULL DEFAULT (datetime('now')),
                source TEXT,
                last_played TEXT,
                launch_path TEXT,
                stored_api_data TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (category_id) REFERENCES categories(category_id) ON DELETE CASCADE
            );
            
            CREATE TABLE IF NOT EXISTS game_tags (
                game_tag_id INTEGER PRIMARY KEY AUTOINCREMENT, 
                game_entry_id INTEGER NOT NULL,
                tag TEXT NOT NULL,
                FOREIGN KEY (game_entry_id) REFERENCES game_entries(game_entry_id) ON DELETE CASCADE,
                UNIQUE(game_entry_id, tag COLLATE NOCASE)
            );
            
            CREATE INDEX IF NOT EXISTS idx_game_entries_category ON game_entries(category_id);
            CREATE INDEX IF NOT EXISTS idx_game_entries_name ON game_entries(name COLLATE NOCASE);
            CREATE INDEX IF NOT EXISTS idx_game_tags_entry ON game_tags(game_entry_id);"
        )
    }
}
